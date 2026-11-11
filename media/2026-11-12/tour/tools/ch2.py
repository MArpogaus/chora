import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *

OUT = "/home/user/log/media/2026-11-12/tour"
L = Launcher("tiling")
stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
open(f"{stub}/foot", "w").write(f"""#!/bin/sh
c={L.dir}/count; n=$(cat $c 2>/dev/null || echo 0); echo $((n+1)) >$c
case $((n % 4)) in 0) k=notes;; 1) k=code;; 2) k=palette;; 3) k=page;; esac
args=""; [ $k = code ] && args={REPO}/crates/chora-kit/src/wobbly.rs
exec /usr/bin/foot --app-id=$k --title=$k -o pad=14x12 -o colors.background=1a1b26 -o colors.foreground=c0caf5 -o 'font=DejaVu Sans Mono:size=9' -o cursor.blink=no -e sh -c "python3 -I {CONTENT} $k $args; exec sleep infinity"
""")
os.chmod(f"{stub}/foot", 0o755)
env = L.start([f"{BIN}/chora-tiling"], flavor="tiling",
  companions=[["swaybg", "--color", "#2e3440"], ["dbus-run-session", "--", "waybar", "--config", f"{REPO}/flavors/tiling/waybar/config.jsonc"], [f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(8)
t = Tour(S, OUT, f"{L.dir}/work")
t.begin("2 tiling", "02-tiling")
try:
    t.jump(640, 360)
    t.wait(2)
    t.act("look", "Look at the empty tiling desktop", "Dark background, waybar at the top with the tag and the clock, the Chat pill", still="02-01-empty")
    t.done()
    for i, name in enumerate(["first", "second", "third", "fourth"], 1):
        t.act("key", f"Press Super+Return ({name} window)", f"A new terminal opens and tiles ({i} window{'s' if i > 1 else ''})", settle=3.0, still=f"02-0{i + 1}-window-{i}")
        t.key("Super+Return"); t.done()
    layouts = ["tile_left", "tile_bottom", "tile_top", "fair", "fair_horizontal"]
    for i, lay in enumerate(layouts):
        t.act("key", "Press Super+space (next layout)", f"The windows re-tile in the layout {lay}", settle=1.5, still=f"02-{7 + i:02d}-layout-{lay}")
        t.key("Super+space"); t.done()
    t.act("key", "Press Super+j (focus next window)", "Focus moves to the next window", settle=1.0, still="02-12-focus-next")
    t.key("Super+j"); t.done()
    t.act("key", "Press Super+Shift+j (swap with next)", "The focused window swaps places", settle=1.2, still="02-13-swap")
    t.key("Super+Shift+j"); t.done()
    t.act("key", "Press Super+l (wider master)", "The master area grows by 5 percent", settle=1.2, still="02-14-master-wider")
    t.key("Super+l"); t.done()
    t.act("key", "Press Super+2 (tag 2)", "An empty tag 2 shows, the bar marks tag 2", settle=1.2, still="02-15-tag-2")
    t.key("Super+2"); t.done()
    t.act("key", "Press Super+1 (back to tag 1)", "The four windows come back", settle=1.2, still="02-16-tag-1")
    t.key("Super+1"); t.done()
    t.flush("bar")
    bar = [(n["id"], n.get("rects")) for n in S.query("kind:surface") if n.get("rects")]
    print(bar)
    t.act("pointer", "Move the pointer over the bar", "The bar stays calm", settle=0.5, still="02-17-bar-hover")
    t.move(1230, 12); t.done()
    t.act("key", "Press Super+a (chat panel)", "The chat opens as a side panel", settle=2.5, still="02-18-chat-panel")
    t.key("Super+a"); t.done()
    msg = "Hello, are you there?"
    t.act("keys", f"Type '{msg}' without a helper picked", "Text shows in the field", settle=0.5, still="02-19-typed")
    t.text(msg); t.done()
    t.act("key", "Press Return", "A hint: pick a helper first", settle=1.5, still="02-20-no-helper")
    t.key("Return"); t.done()
    t.act("key", "Press Escape", "The chat panel closes", settle=1.5, still="02-21-chat-closed")
    t.key("Escape"); t.done()
    t.wait(1)
    t.finish("02-tiling.mp4")
    json.dump(t.rows, open(f"{OUT}/rows-ch2.json", "w"), indent=1)
finally:
    L.stop()
