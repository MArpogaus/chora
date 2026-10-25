#!/bin/bash
# Gate check 10 of 0.8: an MCP client puts a text node under
# screen:NAME, and a capture shows it. The capture must hold
# non-background pixels, so a black rectangle does not pass.
#
# Usage: tests/gate-0.8-10.sh   BIN: directory with the binaries
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
fonts=/tmp/gate-0.8-10-fonts
rm -rf "$fonts" && mkdir "$fonts"
ln -s "$src"/*.tt[fc] "$src"/*.otf "$fonts"/ 2>/dev/null
rm -f "$fonts"/*COLRv1*
export CHORA_FONT_DIRS=$fonts
session_start 800x600
grant screen.capture
out=$(REPO=$repo mcp_py <<'PY' 2>&1
import importlib.util, os, sys, time
from gate_mcp import Mcp

spec = importlib.util.spec_from_file_location(
    "pngpixel", os.path.join(os.environ["REPO"], "tests", "png-pixel.py"))
pngpixel = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pngpixel)

mcp = Mcp()
create = {"id": "label", "kind": "text", "parent": "screen:HEADLESS-1",
          "props": {"text": "Grüße 你好 🙂", "font_size": 32}}
answer = mcp.call("apply_commands", {"commands": [{"create": create}]})
if answer.get("isError"):
    sys.exit(f"the core refuses kind text: {answer['content']}")
nodes = []
for _ in range(50):
    nodes = mcp.json("query_scene", {"selector": "id:label"})
    if nodes and nodes[0].get("rects"):
        break
    time.sleep(0.2)
else:
    sys.exit(f"the text node drew no rect: {nodes}")
mcp.capture_png("capture", {"node": "label"}, "/tmp/gate-0.8-10.png")
with open("/tmp/gate-0.8-10.png", "rb") as f:
    data = f.read()
ink = sum(1 for line in pngpixel.rows(data)
          for r, g, b in (line[x * 4:x * 4 + 3]
                          for x in range(len(line) // 4))
          if r + g + b > 96)
if ink < 40:
    sys.exit(f"the capture of the text node holds {ink} "
             f"non-background pixels")
print(f"the capture of the text node holds {ink} non-background pixels")
PY
)
rc=$?
((rc == 0)) || fail "the MCP client: $out"
echo "$out"
ok "the text node under screen:NAME shows in a capture"
