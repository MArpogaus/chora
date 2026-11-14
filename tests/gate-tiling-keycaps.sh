#!/bin/bash
# The key list of tiling, measured on a capture: the margin left of the first keycap and the margin right of the
# longest word are the same, within 2 px; the text of every keycap,
# descenders included, sits inside the cap with at least 1 px to spare,
# and the first letter of a keycap stands on the same baseline as the
# first letter of the label beside it.
#
# Usage: tests/gate-tiling-keycaps.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture input.inject
echo '[{"shortcut": {"keys": "Super+slash"}}]' | ctl inject - >/dev/null || fail "Super+slash"
sleep 2
shot=$XDG_CONFIG_HOME/keycaps.png
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
ctl query 'kind:rect' >"$XDG_CONFIG_HOME/rects.json" || fail "the query"
ctl query 'kind:text' >"$XDG_CONFIG_HOME/texts.json" || fail "the query"

python3 -I - "$repo/tests/png-pixel.py" "$shot" "$XDG_CONFIG_HOME/rects.json" "$XDG_CONFIG_HOME/texts.json" <<'PY' || fail "the key list is drawn badly"
import importlib.util, json, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
rect = lambda n: [round(v) for v in next(iter(n["rects"].values()))]
nodes = {n["id"]: n for n in json.load(open(sys.argv[3])) if n.get("rects")}
texts = {n["id"]: n for n in json.load(open(sys.argv[4])) if n.get("rects")}
panel = rect(nodes["tiling-keys-panel"])
cap_fill = (0x24, 0x28, 0x36)
panel_fill = px(panel[0] + 6, panel[1] + panel[3] - 6)
far = lambda a, b: sum(abs(p - q) for p, q in zip(a, b)) > 90
problems = []

caps = sorted((rect(n), i) for i, n in nodes.items() if i.endswith("-cap") and i.startswith("tiling-keys"))
labels = {i: rect(n) for i, n in texts.items() if i.endswith("-l") and i.startswith("tiling-keys")}
assert caps and labels, "no caps or labels"
left = min(c[0][0] for c in caps) - panel[0]
best_right = 0
for i, r in labels.items():
    ys = range(r[1], r[1] + r[3])
    xs = [x for x in range(r[0], min(r[0] + r[2] + 8, panel[0] + panel[2] - 1)) if any(far(px(x, y), panel_fill) for y in ys)]
    best_right = max(best_right, max(xs, default=0))
right = panel[0] + panel[2] - 1 - best_right
if abs(left - right) > 2:
    problems.append(f"the margin is {left} px left of the caps and {right} px right of the longest word")

def ink_rows(box, fill):
    return [y for y in range(box[1] - 3, box[1] + box[3] + 4) if any(far(px(x, y), fill) for x in range(box[0] + 3, box[0] + box[2] - 3))]

def label_of(ident):
    prefix, n = ident[: -len("-cap")].rsplit("-", 1)
    return labels[f"{prefix}-{int(n) + 1}-l"]

bad = 0
for box, ident in caps:
    word = texts.get(ident[: -len("-cap")] + "-k")
    ys = [y for y in ink_rows(box, cap_fill) if box[1] <= y < box[1] + box[3]]
    outside = [y for y in ink_rows(box, panel_fill) if not box[1] <= y < box[1] + box[3] and y in range(box[1] - 3, box[1] + box[3] + 3) and px(box[0] + box[2] // 2, y) == panel_fill]
    if word and ys and (min(ys) < box[1] + 1 or max(ys) > box[1] + box[3] - 2):
        bad += 1
        last = f"the text of {word['text']!r} spans y {min(ys)}-{max(ys)} in a cap of y {box[1]}-{box[1] + box[3] - 1}"
if bad:
    problems.append(f"{bad} caps hold text at their edge, for example {last}")

def bottom(x0, y0, y1, fill):
    ys = [y for y in range(y0, y1) if any(far(px(x, y), fill) for x in range(x0, x0 + 5))]
    return max(ys, default=None)

for box, ident in caps:
    word = texts.get(ident[: -len("-cap")] + "-k")
    if word and word["text"].startswith("Super+j"):
        label = label_of(ident)
        cap_base = bottom(rect(word)[0], box[1], box[1] + box[3], cap_fill)
        label_base = bottom(label[0], label[1] - 2, label[1] + label[3] + 2, panel_fill)
        if cap_base != label_base:
            problems.append(f"the cap text stands on y {cap_base}, the label on y {label_base}")
        break
print("margins", left, right)
if problems:
    sys.exit("; ".join(problems))
PY
ok "equal margins in the key list; keycap text inside its cap, on the baseline of its label"
