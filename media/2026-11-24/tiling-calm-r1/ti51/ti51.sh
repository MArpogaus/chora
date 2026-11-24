#!/bin/bash
# ti-51 verify: the contrast of the rows of the key list alone and under a popup.
set -u
repo=/home/user/chora
. "$repo/tests/gate-lib.sh"
need fuzzel
need_bins chora choractl chora-tiling
temp_config
out=${OUT:-/tmp/claude-0/ti51/out}
mkdir -p "$out"
session_start 1280x800 -- "$bin/chora-tiling" --no-companions
grant screen.capture input.inject methods.call
ctl run foot sleep 600 >/dev/null
settle
echo '[{"shortcut": {"keys": "Super+slash"}}]' | ctl inject - >/dev/null
settle
sleep 1
ctl capture --output HEADLESS-1 "$out/keys-alone.png" >/dev/null || fail capture
ctl query 'kind:text' >"$out/texts.json"
printf 'a\nb\nc\n' | ctl run fuzzel --dmenu >/dev/null 2>&1
settle
sleep 1
ctl capture --output HEADLESS-1 "$out/keys-under-popup.png" >/dev/null || fail capture2
python3 -I - "$repo/tests/png-contrast.py" "$out" <<'PY'
import importlib.util, json, sys, pathlib
spec = importlib.util.spec_from_file_location("pc", sys.argv[1]); pc = importlib.util.module_from_spec(spec); spec.loader.exec_module(pc)
out = pathlib.Path(sys.argv[2])
texts = [n for n in json.load(open(out / "texts.json")) if n["id"].startswith("tiling-keys-body") and n.get("rects")]
def rows(name): return list(pc.png.rows((out / name).read_bytes()))
A, B = rows("keys-alone.png"), rows("keys-under-popup.png")
H, W = len(A), len(A[0]) // 4
diff = [(x, y) for y in range(0, H, 2) for x in range(0, W, 2) if A[y][x*4:x*4+3] != B[y][x*4:x*4+3]]
bx0, bx1 = min(x for x, y in diff), max(x for x, y in diff)
by0, by1 = min(y for x, y in diff), max(y for x, y in diff)
print(f"popup box x {bx0}-{bx1} y {by0}-{by1}; key list texts {len(texts)}")
def measure(rows_, n):
    x, y, w, h = (int(v) for v in next(iter(n["rects"].values())))
    return pc.contrast_rows(rows_, max(x, 0), max(y, 0), min(w, W - x), min(h, H - y))[0]
def under(n):
    x, y, w, h = (int(v) for v in next(iter(n["rects"].values())))
    return x >= bx0 and x + w <= bx1 and y >= by0 and y + h <= by1
for label, rws in (("alone", A), ("under popup", B)):
    ratios = [(measure(rws, n), n) for n in texts]
    low = [(r, n) for r, n in ratios if r < 7]
    low1 = [(r, n) for r, n in ratios if r < 1.1]
    inside = [(r, n) for r, n in low if under(n)]
    print(f"{label}: rows {len(ratios)}, under 7:1 {len(low)}, about 1.00:1 {len(low1)}, of the low ones inside the popup box {len(inside)}")
    for r, n in low[:6]:
        print(f"   {r:.2f}:1 {n['text']!r} {n['id']} inside popup box: {under(n)}")
PY
