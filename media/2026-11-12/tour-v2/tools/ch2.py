import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *
from slides import slide

OUT = "/home/user/log/media/2026-11-12/tour-v2"
L = Launcher("tiling")
stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
open(f"{stub}/foot", "w").write(f"""#!/bin/sh
c={L.dir}/count; n=$(cat $c 2>/dev/null || echo 0); echo $((n+1)) >$c
colors="-o colors.background=1a1b26 -o colors.foreground=c0caf5 -o pad=14x12 -o cursor.blink=no"
if [ $n = 0 ]; then exec /usr/bin/foot --app-id=shell --title=shell $colors -o 'font=DejaVu Sans Mono:size=10'; fi
case $((n % 3)) in 1) k=notes;; 2) k=code;; 0) k=page;; esac
args=""; [ $k = code ] && args={REPO}/crates/chora-kit/src/wobbly.rs
exec /usr/bin/foot --app-id=$k --title=$k $colors -o 'font=DejaVu Sans Mono:size=9' -e sh -c "python3 -I {CONTENT} $k $args; exec sleep infinity"
""")
os.chmod(f"{stub}/foot", 0o755)
env = L.start([f"{BIN}/chora-tiling"], flavor="tiling",
  companions=[["swaybg", "--color", "#2e3440"], ["dbus-run-session", "--", "waybar", "--config", f"{REPO}/flavors/tiling/waybar/config.jsonc"], [f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(8)
t = Tour(S, OUT, f"{L.dir}/work")
t.begin("2 tiling", "02-tiling")
O = "HEADLESS-1"
try:
    t.jump(640, 360)
    t.wait(1.2)
    t.act("look", "The empty tiling desktop", "Dark background, waybar with clock and volume, the Chat pill", still="02-01-empty")
    t.done()
    t.act("key", "Press Super+Return", "A terminal opens and fills the screen", settle=2.5, still="02-02-terminal")
    t.key("Super+Return"); t.done()
    t.act("keys", "Type cat friday.md", "The shell prints the Friday list", settle=0.6, still="02-03-cat")
    t.text("cat friday.md\n"); t.done()
    for i, name in enumerate(["second", "third", "fourth"], 2):
        t.act("key", f"Press Super+Return ({name} window)", f"A new window opens as master, {i} windows tile", settle=2.5, still=f"02-0{i + 2}-window-{i}")
        t.key("Super+Return"); t.done()
    for i, lay in enumerate(["tile_left", "tile_bottom", "fair"]):
        t.act("key", "Press Super+space (next layout)", f"The windows re-tile in {lay}" if lay != "fair" else "After two more presses the windows are in a 2x2 grid", settle=1.2, still=f"02-{7 + i:02d}-layout-{lay}")
        if lay == "fair":
            t.key("Super+space"); t.wait(0.4); t.key("Super+space"); t.wait(0.4); t.key("Super+space")
        else:
            t.key("Super+space")
        t.done()
    t.flush("tiles")
    rects = [(n["id"], n["rects"][O]) for n in S.query("kind:surface") if n.get("rects") and O in n["rects"]]
    rects.sort(key=lambda r: (r[1][1], r[1][0]))
    print(rects)
    r = rects[-1][1]
    t.act("pointer", "Move the pointer over another window", "Focus follows the pointer", settle=0.8, still="02-10-focus-pointer")
    t.move(r[0] + r[2] / 2, r[1] + r[3] / 2); t.done()
    t.act("key", "Press Super+l (wider master)", "The master area grows", settle=1.0, still="02-11-master-wider")
    t.key("Super+l"); t.done()
    t.act("key", "Press Super+Shift+c (close the focused window)", "The window closes and the others fill the space", settle=1.5, still="02-12-closed")
    t.key("Super+Shift+c"); t.done()
    t.act("key", "Press Super+Shift+2 (send the window to tag 2)", "The focused window leaves tag 1 for tag 2", settle=1.2, still="02-13-to-tag-2")
    t.key("Super+Shift+2"); t.done()
    t.act("key", "Press Super+2 (view tag 2)", "Tag 2 shows the moved window alone", settle=1.2, still="02-14-tag-2")
    t.key("Super+2"); t.done()
    t.act("key", "Press Super+1 (back to tag 1)", "The other windows are back", settle=1.2, still="02-15-tag-1")
    t.key("Super+1"); t.done()
    t.act("pointer", "Move the pointer over the Connected module of the bar", "A tooltip shows the address", settle=0.6, still="02-16-bar-hover")
    t.move(1230, 12); t.done()
    t.flush("voice")
    L.emit("voice.state", {"state": "listening", "task": "", "said": ""})
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4})
    t.act("key", "Press Super+a (chat panel)", "The chat opens as a side panel with the helper list", settle=2.0, still="02-17-chat-panel")
    t.key("Super+a"); t.done()
    t.act("keys", "Type 1 and press Return (pick Atlas)", "Atlas is picked: header and transcript", settle=1.5, still="02-18-picked")
    t.key("1"); t.wait(0.3); t.key("Return"); t.done()
    msg = "Please also add Monday."
    t.act("keys", f"Type '{msg}' and press Return", "The message is sent and the field empties", settle=1.2, still="02-19-sent")
    t.text(msg); t.key("Return"); t.done()
    t.act("key", "Press Escape", "The chat panel closes", settle=1.2, still="02-20-chat-closed")
    t.key("Escape"); t.done()
    t.wait(0.5)
    slide(f"{L.dir}/slide.png", 2, "tiling", ["Windows that tile, in the style of awesome.", "Open terminals, change layouts, tags, the bar", "and the chat panel."], accent=(158, 206, 106))
    t.finish("02-tiling.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch2.json", "w"), indent=1)
finally:
    L.stop()
