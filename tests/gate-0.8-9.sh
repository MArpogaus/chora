#!/bin/bash
# Gate check 9 of 0.8: a text node shows "Grüße 你好 🙂" with fallback
# fonts. The check captures the node and counts non-background pixels
# in the part of the rect of each word. No reference picture is stored.
#
# Usage: tests/gate-0.8-9.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need python3
# the fallback fonts of the plan: CJK and emoji, outside the repo.
# Noto-COLRv1 stays out: swash lays it out but draws nothing.
src=${CHORA_FONT_DIRS:-$HOME/.local/share/fonts/chora-test}
test -d "$src" || skip "no font directory $src with a font for 你好 and 🙂"
fonts=/tmp/gate-0.8-9-fonts
rm -rf "$fonts" && mkdir "$fonts"
ln -s "$src"/*.tt[fc] "$src"/*.otf "$fonts"/ 2>/dev/null
rm -f "$fonts"/*COLRv1*
export CHORA_FONT_DIRS=$fonts
session_start 800x600
grant screen.capture
out=$(REPO=$repo mcp_py <<'PY' 2>&1
import importlib.util, math, os, sys, time
from gate_mcp import Mcp

FS = 40.0
WORDS = ["Grüße", "你好", "🙂"]
spec = importlib.util.spec_from_file_location(
    "pngpixel", os.path.join(os.environ["REPO"], "tests", "png-pixel.py"))
pngpixel = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pngpixel)

mcp = Mcp()

def create(node_id, text, y):
    mcp.tool("apply_commands", {"commands": [{"create": {
        "id": node_id, "kind": "text", "parent": "screen:HEADLESS-1",
        "props": {"text": text, "font_size": FS,
                  "transform": {"translate": [40, y]}}}}]})

def layout_width(node_id):
    nodes = []
    for _ in range(50):
        nodes = mcp.json("query_scene", {"selector": f"id:{node_id}"})
        for rect in (nodes[0].get("rects") or {}).values() if nodes else []:
            if rect:
                return rect[2]
        time.sleep(0.2)
    sys.exit(f"the text node {node_id} drew no rect: {nodes}")

# one probe node per word measures the word widths of this font set
for i, word in enumerate(WORDS):
    create(f"probe{i}", word, 400 + 60 * i)
widths = [layout_width(f"probe{i}") for i in range(3)]
create("line", " ".join(WORDS), 60)
total = layout_width("line")
mcp.capture_png("capture", {"node": "line"}, "/tmp/gate-0.8-9.png")
space = max((total - sum(widths)) / 2.0, 1.0)
parts = [(0.0, widths[0]),
         (widths[0] + space, widths[0] + space + widths[1]),
         (total - widths[2], total)]
with open("/tmp/gate-0.8-9.png", "rb") as f:
    data = f.read()
ink = [0] * 3
for line in pngpixel.rows(data):
    scale = (len(line) // 4) / total
    for x in range(len(line) // 4):
        r, g, b = line[x * 4:x * 4 + 3]
        if r + g + b <= 96:
            continue
        for i, (left, right) in enumerate(parts):
            edge = (right - left) * 0.1
            if left + edge <= x / scale <= right - edge:
                ink[i] += 1
for word, wide, count in zip(WORDS, widths, ink):
    if count < 40:
        sys.exit(f"{word} (measured {wide:.0f} px) has {count} "
                 f"non-background pixels in its part")
print(f"{int(math.ceil(total))} px wide, ink per word: {ink}")
PY
)
rc=$?
((rc == 0)) || fail "the text node: $out"
echo "$out"
ok "each of Grüße, 你好 and 🙂 has ink in its part of the text node"
