import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from human import Human
from tour import Session
from slides import slide

OUT = os.environ.get("TOUR_OUT", "/home/user/log/media/2026-11-16/tour-v3")
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
t = Human(S, OUT, f"{L.dir}/work")
t.begin("2 tiling", "02-tiling")
O = "HEADLESS-1"
try:
    t.jump(640, 360)
    t.wait(1.5)
    t.act("look", "The person looks at the empty tiling desktop", "Dark ink desktop, the bar with Apps, tags and layout at the left, clock, volume, Connected and Keys", still="02-01-empty", lead=0.2, settle=2.5)
    t.done()
    t.act("pointer", "Click Keys in the bar, the visible door to the key list", "The key list opens", settle=2.0, still="02-02-keys-door")
    t.reach(1248, 20); t.click(); t.wait(1.5); t.done()
    t.act("key", "Escape closes the list", "The key list closes", settle=1.0, still="02-03-keys-closed")
    t.wait(1.0); t.tap("Escape", 0.2); t.done()
    t.act("key", "Slip: press Super+s, which is not a key here", "Nothing happens and nothing is typed into a window", settle=1.0, still="02-04-slip")
    t.key("Super+s"); t.done()
    t.act("key", "Open the key list with Super+slash and read it", "A panel lists every key in groups", settle=2.5, still="02-05-key-list")
    t.key("Super+slash"); t.wait(2.0); t.done()
    t.act("key", "Escape closes the key list", "The key list closes", settle=1.0, still="02-06-key-list-closed")
    t.tap("Escape", 0.2); t.done()
    t.act("key", "Press Super+Return (terminal)", "A terminal opens and fills the screen", settle=2.5, still="02-07-terminal")
    t.key("Super+Return"); t.done()
    t.act("keys", "Type ls and cat friday.md, with a slip corrected", "The shell lists the notes and prints the Friday list", settle=0.8, still="02-08-cat")
    t.type_human("ls\n", typo_at=1, typo_char="x"); t.wait(0.8); t.type_human("cat friday.md\n", think=(0,)); t.done()
    t.act("keys", "Open the file in a text editor: nano friday.md, add a line, save and leave", "The editor shows the file, the new line, then the shell returns", settle=1.0, still="02-09-editor")
    t.type_human("nano friday.md\n", think=(0,)); t.wait(1.5)
    t.key("ctrl+End"); t.type_human("\n- Monday: plan the week"); t.wait(0.6); t.key("ctrl+o"); t.wait(0.8); t.tap("Return", 0.6); t.done()
    t.key("ctrl+x"); t.wait(1.0)
    for i, name in enumerate(["second", "third", "fourth"], 2):
        t.act("key", f"Press Super+Return ({name} window)", f"A new window opens, {i} windows tile", settle=2.5, still=f"02-1{i}-window-{i}")
        t.key("Super+Return"); t.done()
    for i, lay in enumerate(["tile_left", "tile_bottom", "fair"]):
        t.act("key", "Press Super+Shift+space (next layout)", f"The windows re-tile in {lay}" if lay != "fair" else "After two more presses the windows are in a 2x2 grid", settle=1.5, still=f"02-2{i}-layout-{lay}")
        if lay == "fair":
            t.key("Super+Shift+space"); t.wait(0.6); t.key("Super+Shift+space"); t.wait(0.6); t.key("Super+Shift+space")
        else:
            t.key("Super+Shift+space")
        t.done()
    t.flush("tiles")
    rects = [(n["id"], n["rects"][O]) for n in S.query("kind:surface") if n.get("rects") and O in n["rects"]]
    rects.sort(key=lambda r: (r[1][1], r[1][0]))
    r = rects[-1][1]
    t.act("pointer", "Move the pointer over another window", "Focus follows the pointer; the focused window has the gradient border", settle=0.8, still="02-30-focus-pointer")
    t.reach(r[0] + r[2] / 2, r[1] + r[3] / 2); t.done()
    t.act("key", "Press Super+l (wider master)", "The master area grows", settle=1.0, still="02-31-master-wider")
    t.key("Super+l"); t.done()
    t.act("key", "Press Super+Shift+c (close the focused window)", "The window closes and the others fill the space", settle=1.5, still="02-32-closed")
    t.key("Super+Shift+c"); t.done()
    t.act("key", "Press Super+Shift+2 (send the window to tag 2)", "The focused window leaves tag 1 for tag 2", settle=1.2, still="02-33-to-tag-2")
    t.key("Super+Shift+2"); t.done()
    t.act("key", "Press Super+2 (view tag 2)", "Tag 2 shows the moved window alone", settle=1.2, still="02-34-tag-2")
    t.key("Super+2"); t.done()
    t.act("key", "Press Super+1 (back to tag 1)", "The other windows are back", settle=1.2, still="02-35-tag-1")
    t.key("Super+1"); t.done()
    t.flush("voice")
    L.emit("voice.state", {"state": "listening", "task": "", "said": ""})
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4})
    t.act("pointer", "Click Chat ... at the top right, the visible door", "The chat opens as a side panel with the helper list", settle=2.0, still="02-37-chat-panel")
    t.reach(1216, 80); t.click(); t.done()
    t.act("keys", "Type 1 and press Return (pick Atlas)", "Atlas is picked: header and transcript", settle=1.5, still="02-38-picked")
    t.key("1"); t.wait(0.3); t.key("Return"); t.done()
    msg = "Please also add Monday."
    t.act("keys", f"Type '{msg}' and press Return", "The message is sent and the field empties", settle=1.2, still="02-39-sent")
    t.type_human(msg, typo_at=7, typo_char="k"); t.wait(0.5); t.key("Return"); t.done()
    t.act("key", "Press Escape", "The chat panel closes", settle=1.2, still="02-40-chat-closed")
    t.key("Escape"); t.done()
    t.wait(0.5)
    slide(f"{L.dir}/slide.png", 2, "tiling", ["Tiling v2: lit glass on ink.", "Open terminals, layouts, a gradient focus border,", "tags, the bar, the key list and the chat panel."], accent=(158, 206, 106))
    t.finish("02-tiling.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch2.json", "w"), indent=1)
finally:
    L.stop()
