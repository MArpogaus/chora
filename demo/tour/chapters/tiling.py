"""Chapter tiling: one function `run(ctx)`; the rows are `t.act(..., expect=, contract=, cells=, capture=)`."""

import sys, time, os, json, subprocess, re, shutil
from launch import *
import launch
from human import Quick
from tour import Session, checker_for
from harness import Expect
from slides import slide
from chapters.checks import Diff, shown, gone, both, logged, notice_is_new, moved, fuzzel_log, box_says, rows_have, volume_is, first_rows

NOTES = "/home/user/notes"
O = "HEADLESS-1"


def windows_of(after, look="tiling"):
    """The windows of the look that are shown."""
    return [w for w in (after.states.get(look) or {}).get("windows", []) if w.get("shown")]


def windows_are(n, look="tiling"):
    """A check: the look shows n windows."""
    return lambda after: [] if len(windows_of(after, look)) == n else [f"{look} shows {len(windows_of(after, look))} windows, expected {n}"]


def tiles_hold(n, look="tiling"):
    """A check (L1, tiling-audit): n tiled windows, none over another, one gap between neighbours."""

    def check(after):
        rects = [w["rect"] for w in windows_of(after, look) if not w.get("floating")]
        out = [] if len(rects) == n else [f"{len(rects)} tiled windows, expected {n}"]
        gaps = set()
        for a in rects:
            right = [b[0] - (a[0] + a[2]) for b in rects if b is not a and b[1] < a[1] + a[3] - 0.5 and a[1] < b[1] + b[3] - 0.5 and b[0] >= a[0] + a[2] - 0.5]
            below = [b[1] - (a[1] + a[3]) for b in rects if b is not a and b[0] < a[0] + a[2] - 0.5 and a[0] < b[0] + b[2] - 0.5 and b[1] >= a[1] + a[3] - 0.5]
            gaps |= {round(min(g)) for g in (right, below) if g}
            out += [f"windows overlap: {a} {b}" for b in rects if b is not a and b[0] < a[0] + a[2] - 0.5 and a[0] < b[0] + b[2] - 0.5 and b[1] < a[1] + a[3] - 0.5 and a[1] < b[1] + b[3] - 0.5 and a < b]
        return out + ([f"neighbours have unequal gaps {sorted(gaps)}"] if len(gaps) > 1 else [])

    return check


def master_is_focused(after, look="tiling"):
    """The focused window is the largest tile: the master."""
    state = after.states.get(look) or {}
    area = lambda w: w["rect"][2] * w["rect"][3]
    tiles = [w for w in windows_of(after, look) if not w.get("floating")]
    focus = (state.get("focus") or {}).get("surface")
    biggest = max(tiles, key=area)["surface"] if tiles else None
    return [] if focus == biggest else [f"the focus is {focus}, the master is {biggest}"]


def master_wider(before, after, look="tiling"):
    """A delta: the master (the largest tile) is wider than before."""
    width = lambda s: max((w["rect"][2] for w in windows_of(s, look) if not w.get("floating")), default=0)
    return [] if width(after) > width(before) else [f"the master is {width(after)} wide, was {width(before)}"]


def layout_changed(before, after, look="tiling"):
    """A delta: the tiles lie elsewhere than before."""
    rects = lambda s: {w["surface"]: tuple(w["rect"]) for w in windows_of(s, look)}
    return [] if rects(before) != rects(after) else ["no tile moved: the layout did not change"]


def on_tag(tag, count, look="tiling"):
    """A check: `count` windows are on that tag and hidden, the view shows tag 1 (L3)."""

    def check(after):
        state = after.states.get(look) or {}
        away = [w["surface"] for w in state.get("windows", []) if w.get("desks") == [tag] and not w.get("shown")]
        viewed = [d.get("shown") for d in state.get("desks", [])]
        return ([] if len(away) == count else [f"{len(away)} windows on tag {tag}, expected {count}"]) + ([] if viewed == [[1]] else [f"the view is {viewed}, expected tag 1"])

    return check


def floating_on_top(after, look="tiling"):
    """Exactly one window floats and it is on top."""
    wins = windows_of(after, look)
    floats = [w for w in wins if w.get("floating")]
    top = max(wins, key=lambda w: w["z"])["surface"] if wins else None
    if len(floats) != 1:
        return [f"{len(floats)} floating windows, expected 1"]
    return [] if floats[0]["surface"] == top else [f"the floating window {floats[0]['surface']} is under {top}"]


def ran(history, command):
    """A check: the shell of the first terminal ran the command (HISTFILE of the stub)."""

    def check(after):
        lines = open(history).read().splitlines() if os.path.exists(history) else []
        return [] if command in lines else [f"the shell never ran {command!r}: {lines[-5:]}"]

    return check


def file_has(path, text):
    """A check: the file has the text."""
    return lambda after: [] if os.path.exists(path) and text in open(path).read() else [f"{path} lacks {text!r}"]


def probe_heard_no_key(report, chords):
    """A check (T5, tests/persona/check.py verify_probe): the probe window is in the scene and focused,
    and its report has no `key` line of the last key of a Super chord (evdev codes of `chords`). The
    Super key itself (125) reaches it; the contract does not hold that."""

    def check(after):
        probes = [n for n in after.nodes if (n.get("surface") or {}).get("app_id") == "chora-probe"]
        if not probes:
            return ["no chora-probe window in the scene"]
        if not any(n.get("focused") for n in probes):
            return ["the probe window has no focus: the check would be vacuous"]
        lines = open(report).read().splitlines() if os.path.exists(report) else None
        if lines is None:
            return [f"no report {report}"]
        return [f"the probe window got {line!r}, the key of a Super chord" for line in lines if line in {f"key {c}" for c in chords}]

    return check


def chat_beside(after, look="tiling"):
    """The chat covers no tiled window (ti-29)."""
    chat = after.by_id().get("hud-chat")
    rect = next(iter((chat or {}).get("rects", {}).values()), None)
    if rect is None:
        return ["no chat on screen"]
    hit = [w["surface"] for w in windows_of(after, look) if not w.get("floating") and min(rect[0] + rect[2], w["rect"][0] + w["rect"][2]) > max(rect[0], w["rect"][0]) and min(rect[1] + rect[3], w["rect"][1] + w["rect"][3]) > max(rect[1], w["rect"][1])]
    return [f"the chat covers the tiled windows {hit}"] if hit else []


def run(ctx):
    OUT = ctx.out
    L = Launcher("tiling4")
    stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
    history = f"{L.dir}/history"
    report = f"{L.dir}/probe.report"
    shutil.rmtree(f"{NOTES}/demo", ignore_errors=True)
    fuzzel_log(L.dir, stub)
    open(f"{stub}/foot", "w").write(foot_stub(L.dir, history, SAMPLE))
    os.chmod(f"{stub}/foot", 0o755)
    env = L.start(tiling_argv(f"{BIN}/chora-tiling"), flavor="tiling",
      extra_config={"chora-tiling.yaml": "effects: [wobbly]\n"},
      companions=[["swaybg", "--color", "#2e3440"], ["dbus-run-session", "--", "sh", f"{REPO}/flavors/tiling/waybar/waybar.sh", f"{REPO}/flavors/tiling/waybar"], [f"{BIN}/chora-assistant"], [f"{BIN}/chora-notify"]])
    S = Session(env, L.pids, BIN)
    time.sleep(8)
    L.emit("voice.state", {"name": "Atlas", "state": "idle", "task": "", "seq": 0})
    t = Quick(S, OUT, f"{L.dir}/work", checker=checker_for(S, ["tiling"], L.dir))
    t.begin("2 tiling: a hacker builds a project", "02-tiling")
    wpctl_state = f"{stub}/wpctl.state"
    try:
        t.jump(640, 360)
        t.act("look", "The hacker looks at the empty tiling desktop", "Dark desktop and the bar: layout, tags, clock, volume, Keys", scene="t1", still="02-01-empty", lead=0.1, settle=1.0, contract="L2", cells=["gate-tiling-gaps.sh"], capture="Q", expect=Expect(verify=windows_are(0)))
        t.done()
        t.act("key", "Super+Return: a shell", "A terminal fills the screen", scene="t1", still="02-02-shell", settle=0.5, contract="T1", cells=["act: a window opens"], capture="Q", expect=Expect(new_window={"owner": "tiling", "focused": True}, focus="tiling", verify=tiles_hold(1)))
        t.key("Super+Return"); t.cut(1.0); t.wait(0.3); t.done()
        t.act("keys", "Types ls and cat friday.md, one typo", "The listing and the file", scene="t1", still="02-02a-ls", settle=0.5, contract="T1", cells=["act: a window opens"], capture="Q", expect=Expect(verify=both(ran(history, "ls"), ran(history, "cat friday.md"))))
        t.type_quick("ls\n", typo_at=1, typo_char="x"); t.wait(0.2); t.type_quick("cat friday.md\n"); t.wait(0.3); t.done()
        t.act("keys", "Starts a small project: mkdir, a Python file in nano, run it", "The editor shows two lines, the run prints the result", scene="t1", still="02-02b-project", settle=0.8, contract="T1", cells=["act: a window opens"], capture="Q", expect=Expect(verify=both(file_has(f"{NOTES}/demo/main.py", "print('build', n)"), ran(history, "python3 main.py"), windows_are(1))))
        t.type_quick("mkdir demo && cd demo && nano main.py\n"); t.cut(0.8)
        t.type_quick("for n in range(3):\n    print('build', n)\n", typo_at=6, typo_char="m")
        t.wait(0.3); t.key("ctrl+o"); t.wait(0.4); t.tap("Return", 0.4); t.key("ctrl+x"); t.wait(0.4)
        t.type_quick("python3 main.py\n"); t.wait(0.4); t.done()
        t.flush("probe")
        pid_file = f"{L.dir}/probe.pid"
        S.ctl("run", "setsid", "sh", "-c", f"echo $$ >{pid_file}; exec chora-probe window 40 120 200 --report >>{report} 2>&1")
        t.act("keys", "Super+d and two more Super chords while a window of a program listens", "A second window listens beside the editor; no key of a chord reaches it", scene="t2", still="02-02d-stray", settle=0.6, contract="T5", cells=["gate-injected-presses.sh"], capture="Q", expect=Expect(verify=probe_heard_no_key(report, (32, 45, 50))))
        t.wait(0.8)
        for chord in ("Super+d", "Super+x", "Super+m"):
            t.key(chord); t.wait(0.3)
        t.done()
        t.flush("probe done")
        os.kill(int(open(pid_file).read()), 15)
        t.wait(0.5)
        for i, name in enumerate(["code", "notes", "page"], 2):
            t.act("key", f"Super+Return ({name})", f"A window opens and the windows re-tile ({i} windows)", scene="t3", still=f"02-0{i + 1}-window-{i}", settle=0.5, lead=0.05, contract="T1", cells=["act: a window opens"], capture="Q", expect=Expect(new_window={"owner": "tiling", "focused": True}, verify=both(windows_are(i), tiles_hold(i))))
            t.key("Super+Return"); t.cut(0.5); t.wait(0.3); t.done()
        t.act("key", "Super+j moves focus down the stack, Super+Ctrl+Return makes it the master", "The focus mark moves; the window becomes the master", scene="t3", still="02-19-master", settle=0.6, lead=0.05, contract="L2", cells=["gate-tiling-gaps.sh"], capture="Q", expect=Diff(focus="tiling", verify=both(master_is_focused, tiles_hold(4)), delta=layout_changed))
        t.key("Super+j"); t.wait(0.5); t.key("Super+j"); t.wait(0.5); t.key("Super+Ctrl+Return"); t.wait(0.5); t.done()
        t.act("key", "Super+l widens the master", "The master area grows", scene="t3", still="02-20-wider", settle=0.6, lead=0.05, contract="L2", cells=["tiling-audit.py"], capture="Q", expect=Diff(verify=tiles_hold(4), delta=master_wider))
        t.key("Super+l"); t.wait(0.3); t.key("Super+l"); t.done()
        for lay in ("tile_left", "tile_bottom", "fair"):
            t.act("key", "Super+Shift+space (next layout)", f"The four windows re-tile: {lay}", scene="t3", still=f"02-1-{lay}", settle=0.6, lead=0.05, contract="L1", cells=["gate-tiling-gaps.sh", "tiling-audit.py"], capture="Q", expect=Diff(focus="tiling", verify=tiles_hold(4), delta=layout_changed))
            if lay == "fair":
                t.key("Super+Shift+space"); t.wait(0.4); t.key("Super+Shift+space"); t.wait(0.4); t.key("Super+Shift+space")
            else:
                t.key("Super+Shift+space")
            t.wait(0.5); t.done()
        t.act("key", "Super+Shift+2 sends the focused window to tag 2, Super+2 views it, Super+1 comes back", "The window leaves, tag 2 shows it alone, tag 1 has the rest", scene="t4", still="02-21-tag-2", settle=0.4, lead=0.05, contract="L3", cells=["gate-0.9-tiling-tags.sh"], capture="Q", expect=Expect(focus="tiling", verify=both(on_tag(2, 1), windows_are(3))))
        t.key("Super+Shift+2"); t.wait(0.5); t.key("Super+2"); t.wait(0.9); t.still("02-21-tag-2"); t.key("Super+1"); t.wait(0.6); t.done()
        t.flush("float")
        rects = [(n["id"], n["rects"][O]) for n in S.query("kind:surface") if n.get("rects") and O in n["rects"]]
        rects.sort(key=lambda r: (r[1][1], r[1][0]))
        r = rects[0][1]
        cx, cy = r[0] + r[2] / 2, r[1] + r[3] / 2
        t.act("key", "Moves onto a window, Super+Ctrl+space floats it", "The window floats on top of the tiles", scene="t5", still="02-22-floating", settle=0.5, lead=0.05, contract="L2", cells=["gate-tiling-wobbly.sh"], capture="Q", expect=Expect(verify=floating_on_top))
        t.reach(cx, cy); t.key("Super+Ctrl+space"); t.wait(0.6); t.done()
        t.act("pointer", "Super+drag moves the floating window", "The window bends like jelly while it moves", scene="t5", still=None, settle=0.2, lead=0.05, contract="L2", cells=["gate-tiling-wobbly.sh"], capture="H50", expect=Diff(verify=floating_on_top, delta=moved("tiling")))
        t.drag(cx, cy, cx + 300, cy + 120, seconds=1.0, mods=("Super_L",), hold=0.1, mid_still="02-23-wobble")
        t.wait(0.4); t.done()
        t.act("key", "Super+F1 opens the key list", "A panel lists every key in groups", scene="t6", still="02-24-key-list", settle=1.2, lead=0.05, contract="T2", cells=["doors keys-key"], capture="Q", opens=["tiling-keys"], expect=Expect(open=["tiling-keys"]))
        t.key("Super+F1"); t.done()
        t.act("key", "Escape closes it", "The list closes", scene="t6", still=None, settle=0.2, lead=0.05, contract="T2", cells=["act: Escape shuts the key list of the keys key"], capture="Q", closes=["tiling-keys"], expect=Expect(open=[]))
        t.tap("Escape", 0.2); t.done()
        t.act("key", "Super+slash opens the key list once more", "The same panel", scene="t6", still=None, settle=0.8, lead=0.05, contract="T2", cells=["doors keys-alias"], capture="Q", opens=["tiling-keys"], expect=Expect(open=["tiling-keys"]))
        t.key("Super+slash"); t.done()
        t.act("key", "Escape closes it", "The list closes", scene="t6", still=None, settle=0.2, lead=0.05, contract="T2", cells=["act: Escape shuts the key list of the keys alias"], capture="Q", closes=["tiling-keys"], expect=Expect(open=[]))
        t.tap("Escape", 0.2); t.done()
        t.act("key", "Super+Print: a screenshot of the rice", "A notice says the screenshot is saved (or says why not)", scene="t7", still="02-25-screenshot", settle=1.4, lead=0.05, contract="T4:screenshot", cells=["doors screenshot-key", "gate-tiling-print-notice.sh"], capture="Q+0.5", expect=Diff(delta=notice_is_new))
        t.key("Super+Print"); t.wait(1.2); t.done()
        t.flush("shot")
        S.dismiss_notices()
        t.act("pointer", "The beginner clicks Apps in the bar, the visible door to programs, and types frobnicate", "The launcher opens over the tiles and says Nothing found for the word", scene="t8", still="02-26-apps", settle=0.8, contract="T6", cells=["doors launcher-door", "gate-tiling-apps-click.sh"], capture="Q", expect=Expect(verify=box_says(L.dir, "frobnicate")))
        t.reach(28, 12); t.click(); t.wait(1.0)
        t.type_quick("frobnicate"); t.tap("Return", 1.0); t.done()
        t.act("keys", "Types Foot in the launcher and starts it", "A terminal opens beside the tiles", scene="t8", still="02-27-foot", settle=1.0, contract="T1", cells=["doors launcher-door"], capture="Q", expect=Expect(new_window={"owner": "tiling", "focused": True}, verify=windows_are(4)))
        t.type_quick("Foot"); t.tap("Return", 1.0); t.cut(1.0); t.done()
        t.act("key", "Super+a: the chat of the helper beside the tiles", "The chat opens beside the tiles, the editor stays visible", scene="t9", still="02-28-chat", settle=0.8, contract="T3", cells=["doors helper-key", "act: the chat of the helper opens"], capture="Q", known="expect: ti-29 the chat covers the tiled windows; design/tour-v9.md t9 holds that it stands beside them", expect=Expect(helper="idle", verify=both(shown("hud-chat-close"), chat_beside)))
        t.key("Super+a"); t.wait(0.6); t.done()
        t.act("key", "Escape closes the chat", "The chat closes", scene="t9", still=None, settle=0.4, lead=0.05, contract="T3", cells=["act: the chat of the helper shuts"], capture="Q", expect=Expect(verify=gone("hud-chat-close")))
        t.tap("Escape", 0.3); t.done()
        t.flush("volume")
        t.act("pointer", "Scrolls on the volume word of the bar", "The volume goes up by one step (40 to 45 percent)", scene="t10", still="02-29-volume", settle=1.0, lead=0.05, contract="T4:volume", cells=["doors volume-door"], capture="Q", expect=Expect(verify=volume_is(45, wpctl_state)))
        t.reach(1018, 24); t.wait(0.5); t.inject([{"scroll": {"dy": -15}}]); t.wait(0.8); t.done()
        t.act("pointer", "Clicks the power icon at the right end of the bar", "A menu opens: Lock, Log out, Restart, Shut down, Screenshot", scene="t10", still="02-30-power-menu", settle=1.0, lead=0.05, contract="T4:power", cells=["doors power-door", "gate-tiling-power.sh"], capture="Q", expect=Expect(verify=rows_have(L.dir, ["Lock", "Log out", "Restart", "Shut down", "Screenshot"])))
        t.reach(1247, 24); t.click(); t.wait(0.8); t.done()
        PX, PY = float(os.environ.get("TOUR_SHOT_X", "640")), float(os.environ.get("TOUR_SHOT_Y", "476"))
        t.act("pointer", "Clicks the row Screenshot", "The menu closes; a notice says the screenshot is saved", scene="t10", still="02-31-power-screenshot", settle=1.4, lead=0.05, contract="T4:screenshot", cells=["gate-tiling-print-notice.sh"], capture="Q+0.5", expect=Diff(delta=notice_is_new))
        t.reach(PX, PY); t.click(); t.wait(1.6); t.done()
        slide(f"{L.dir}/slide.png", 2, "tiling: a project in tiles", ["A hacker opens a shell and three windows,", "re-tiles, uses tags, floats a window and keeps a screenshot."], accent=(158, 206, 106))
        t.finish("02-tiling.mp4", slide=f"{L.dir}/slide.png")
        json.dump(t.rows, open(f"{OUT}/rows-tiling.json", "w"), indent=1)
    finally:
        L.stop()
