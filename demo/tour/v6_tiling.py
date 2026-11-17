import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour6 import Quick, OUT
from tour import Session
from slides import slide

L = Launcher("tiling4")
stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
open(f"{stub}/foot", "w").write(f"""#!/bin/sh
c={L.dir}/count; n=$(cat $c 2>/dev/null || echo 0); echo $((n+1)) >$c
colors="-o colors.background=1a1b26 -o colors.foreground=c0caf5 -o pad=14x12 -o cursor.blink=no"
if [ $n = 0 ]; then exec /usr/bin/foot --app-id=shell --title=shell $colors -o 'font=DejaVu Sans Mono:size=10'; fi
case $((n % 3)) in 1) k=code;; 2) k=notes;; 0) k=page;; esac
args=""; [ $k = code ] && args={REPO}/crates/chora-kit/src/wobbly.rs
exec /usr/bin/foot --app-id=$k --title=$k $colors -o 'font=DejaVu Sans Mono:size=9' -e sh -c "python3 -I {CONTENT} $k $args; exec sleep infinity"
""")
os.chmod(f"{stub}/foot", 0o755)
env = L.start([f"{BIN}/chora-tiling"], flavor="tiling",
  extra_config={"chora-tiling.yaml": "effects: [wobbly]\n"},
  companions=[["swaybg", "--color", "#2e3440"], ["dbus-run-session", "--", "sh", f"{REPO}/flavors/tiling/waybar/waybar.sh", f"{REPO}/flavors/tiling/waybar"], [f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(8)
t = Quick(S, OUT, f"{L.dir}/work")
t.begin("2 tiling: a hacker builds a project", "02-tiling")
O = "HEADLESS-1"
try:
    t.jump(640, 360)
    t.act("look", "The hacker looks at the empty tiling desktop", "Dark desktop and the bar: layout, tags, clock, volume, Keys", still="02-01-empty", lead=0.1, settle=1.0)
    t.done()
    t.act("key", "Super+Return: a shell. Types ls and cat friday.md, one typo", "A terminal fills the screen; the listing and the file", still="02-02-shell", settle=0.5)
    t.key("Super+Return"); t.cut(1.0); t.wait(0.3)
    t.type_quick("ls\n", typo_at=1, typo_char="x"); t.wait(0.2); t.type_quick("cat friday.md\n"); t.done()
    t.act("keys", "Starts a small project: mkdir, a Python file in nano, run it", "The editor shows three lines, the run prints the result", still="02-02b-project", settle=0.8)
    t.type_quick("mkdir demo && cd demo && nano main.py\n"); t.cut(0.8)
    t.type_quick("for n in range(3):\n    print('build', n)\n", typo_at=6, typo_char="m")
    t.wait(0.3); t.key("ctrl+o"); t.wait(0.4); t.tap("Return", 0.4); t.key("ctrl+x"); t.wait(0.4)
    t.type_quick("python3 main.py\n"); t.wait(0.4); t.done()
    t.act("pointer", "Clicks Apps in the bar, the visible door to programs", "The launcher opens over the shell", still="02-02c-apps", settle=0.8)
    t.reach(28, 12); t.click(); t.wait(0.6); t.done()
    t.act("key", "Escape closes the launcher", "The launcher closes", still=None, settle=0.2, lead=0.05)
    t.tap("Escape", 0.2); t.done()
    for i, name in enumerate(["code", "notes", "page"], 2):
        t.act("key", f"Super+Return ({name})", f"A window opens and the windows re-tile ({i} windows)", still=f"02-0{i + 1}-window-{i}", settle=0.5, lead=0.05)
        t.key("Super+Return"); t.cut(0.5); t.wait(0.3); t.done()
    for lay in ("tile_left", "tile_bottom", "fair"):
        t.act("key", "Super+Shift+space (next layout)", f"The four windows re-tile: {lay}", still=f"02-1-{lay}", settle=0.6, lead=0.05)
        if lay == "fair":
            t.key("Super+Shift+space"); t.wait(0.4); t.key("Super+Shift+space"); t.wait(0.4); t.key("Super+Shift+space")
        else:
            t.key("Super+Shift+space")
        t.wait(0.5); t.done()
    t.act("key", "Super+j moves focus down the stack, Super+Ctrl+Return makes it the master", "The focus mark moves; the window becomes the master", still="02-19-master", settle=0.6, lead=0.05)
    t.key("Super+j"); t.wait(0.5); t.key("Super+j"); t.wait(0.5); t.key("Super+Ctrl+Return"); t.wait(0.5); t.done()
    t.act("key", "Super+l widens the master", "The master area grows", still="02-20-wider", settle=0.6, lead=0.05)
    t.key("Super+l"); t.wait(0.3); t.key("Super+l"); t.done()
    t.act("key", "Super+Shift+2 sends the focused window to tag 2, Super+2 views it, Super+1 comes back", "The window leaves, tag 2 shows it alone, tag 1 has the rest", still="02-21-tag-2", settle=0.4, lead=0.05)
    t.key("Super+Shift+2"); t.wait(0.5); t.key("Super+2"); t.wait(0.9); t.still("02-21-tag-2"); t.key("Super+1"); t.wait(0.6); t.done()
    t.flush("float")
    rects = [(n["id"], n["rects"][O]) for n in S.query("kind:surface") if n.get("rects") and O in n["rects"]]
    rects.sort(key=lambda r: (r[1][1], r[1][0]))
    r = rects[0][1]
    cx, cy = r[0] + r[2] / 2, r[1] + r[3] / 2
    t.act("key", "Moves onto a window, Super+Ctrl+space floats it", "The window floats on top of the tiles", still="02-22-floating", settle=0.5, lead=0.05)
    t.reach(cx, cy); t.key("Super+Ctrl+space"); t.wait(0.6); t.done()
    t.act("pointer", "Super+drag moves the floating window", "The window bends like jelly while it moves", still=None, settle=0.2, lead=0.05)
    t.drag(cx, cy, cx + 300, cy + 120, seconds=1.0, mods=("Super_L",), hold=0.1, mid_still="02-23-wobble")
    t.wait(0.4); t.done()
    t.act("key", "Super+slash opens the key list", "A panel lists every key in groups", still="02-24-key-list", settle=1.2, lead=0.05)
    t.key("Super+slash"); t.done()
    t.act("key", "Escape closes it", "The list closes", still=None, settle=0.2, lead=0.05)
    t.tap("Escape", 0.2); t.done()
    t.act("key", "Super+Print: a screenshot of the rice", "A notice says the screenshot is saved (or says why not)", still="02-25-screenshot", settle=1.4, lead=0.05)
    t.key("Super+Print"); t.done()
    slide(f"{L.dir}/slide.png", 2, "tiling: a project in tiles", ["A hacker opens a shell and three windows,", "re-tiles, uses tags, floats a window and keeps a screenshot."], accent=(158, 206, 106))
    t.finish("02-tiling.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch2.json", "w"), indent=1)
finally:
    L.stop()
