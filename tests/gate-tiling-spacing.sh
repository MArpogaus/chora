#!/bin/bash
# The spacing of the tiling bar (tiling-v2 review r2, 3, 4 and 5): in the
# left island the layout word starts 12 px after the separator, left
# aligned; "Keys" is drawn like "Apps" (the colour of the text, the same
# weight); and every gap between two items of the islands, from the edge
# of an island to the first item, from item to separator, from separator
# to item, is the same 12 px, within 2 px for the side bearing of a glyph.
# The tooltips of Apps and Keys stay inside the islands (r2, 12), and the
# month of the clock marks today (r2, 14).
#
# Usage: tests/gate-tiling-spacing.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling
export PATH=$bin:$PATH
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
CHORA_FLAVOR_DIR=$flavor setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
run_pid $!
grant screen.capture
shot=$XDG_CONFIG_HOME/spacing.png
for _ in $(seq 40); do
    sleep 0.5
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || continue
    [ "$(pixel "$shot" 400 24)" != "$(pixel "$shot" 400 300)" ] && [ "$(pixel "$shot" 1200 24)" != "$(pixel "$shot" 1200 300)" ] && break
done
sleep 1
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"

python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY' || fail "the spacing of the bar"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
far = lambda a, b: sum(abs(p - q) for p, q in zip(a, b)) > 40
problems = []

def islands():
    bg, runs, start = px(640, 300), [], None
    for x in range(0, 1280):
        on = far(px(x, 24), bg)
        if on and start is None:
            start = x
        if not on and start is not None:
            runs.append((start, x - 1))
            start = None
    return [r for r in runs if r[1] - r[0] > 40]

left, centre, right = islands()
def separators(a, b):
    found = []
    for x in range(a + 4, b - 3):
        column = [px(x, y) for y in range(17, 31)]
        if all(c == column[0] for c in column) and sum(abs(p - q) for p, q in zip(column[0], (58, 64, 84))) < 12 and px(x, 12) != column[0]:
            found.append(x)
    return found

def ink(a, b, fill, rows_=range(14, 36)):
    return [x for x in range(a, b + 1) if any(far(px(x, y), fill) for y in rows_)]

def groups(columns, gap):
    out = []
    for x in columns:
        if out and x - out[-1][1] <= gap:
            out[-1][1] = x
        else:
            out.append([x, x])
    return out

fill = px(left[0] + 30, 10)
seps = separators(*left)
if len(seps) != 2:
    sys.exit(f"{len(seps)} separators in the left island: {seps}")
sep1, sep2 = seps
apps = groups(ink(left[0] + 3, sep1 - 2, fill), 10)
words = groups(ink(sep2 + 2, left[1] - 3, fill), 10)
accent = next(x for x in range(sep1 + 1, sep2) if far(px(x, 24), fill) and px(x, 24)[2] > 200)
tag9 = groups(ink(sep2 - 40, sep2 - 2, fill), 3)[-1]
centre9 = (tag9[0] + tag9[1]) / 2
gaps = {
    "left edge to Apps icon": apps[0][0] - left[0] - 1,
    "Apps to the separator": sep1 - apps[-1][1] - 1,
    "separator to tag 1": accent - sep1 - 1,
    "tag 9 to the separator": sep2 - (centre9 + 12) - 1,
    "separator to the layout word": words[0][0] - sep2 - 1,
}
fill_r = px(right[0] + 30, 10)
seps_r = separators(*right)
if len(seps_r) != 1:
    sys.exit(f"{len(seps_r)} separators in the right island: {seps_r}")
sep3 = seps_r[0]
before = groups(ink(right[0] + 3, sep3 - 2, fill_r), 9)
after = groups(ink(sep3 + 2, right[1] - 3, fill_r), 10)
gaps["right edge to the first item"] = before[0][0] - right[0] - 1
for n in range(len(before) - 1):
    gaps[f"item {n + 1} to item {n + 2} before the separator"] = before[n + 1][0] - before[n][1] - 1
gaps["last item to the separator"] = sep3 - before[-1][1] - 1
gaps["separator to Keys"] = after[0][0] - sep3 - 1
for n in range(len(after) - 1):
    gaps[f"Keys group item {n + 1} to {n + 2}"] = after[n + 1][0] - after[n][1] - 1
gaps["last glyph to the right edge"] = right[1] - after[-1][1] - 1
for name, value in gaps.items():
    if abs(value - 12) > 2:
        problems.append(f"{name}: {value} px, not 12")
print("gaps:", gaps)
if gaps["separator to the layout word"] < 10:
    problems.append("the layout word is not 12 px from the separator")
if len(words) != 1:
    problems.append("the layout word is not one run: " + str(words))

def brightest(a, b):
    best = (0, 0, 0)
    for x in range(a, b + 1):
        for y in range(14, 36):
            if sum(px(x, y)) > sum(best):
                best = px(x, y)
    return best

apps_word = brightest(apps[-1][0] - 40, apps[-1][1])
keys_word = brightest(after[0][1] - 48, after[0][1])
if any(abs(p - q) > 12 for p, q in zip(apps_word, keys_word)):
    problems.append(f"Keys is {keys_word}, Apps is {apps_word}: not drawn alike")
if problems:
    sys.exit("; ".join(problems))
PY
ok "12 px between the items of the islands, the layout word left aligned, Keys like Apps"

grant input.inject
for target in 50 1230; do
    echo "[{\"pointer_move\": {\"position\": [$target, 24]}}]" | ctl inject - >/dev/null || fail "the pointer move"
    sleep 0.3
    echo "[{\"pointer_move\": {\"position\": [$((target + 2)), 25]}}]" | ctl inject - >/dev/null || fail "the pointer move"
    sleep 2.5
    ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
    python3 -I - "$repo/tests/png-pixel.py" "$shot" "$target" <<'PY' || fail "the tooltip at $target does not sit inside its island"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
bg = px(640, 300)
far = lambda a, b: sum(abs(p - q) for p, q in zip(a, b)) > 20
cols = [x for x in range(0, 1280) if any(far(px(x, y), bg) for y in range(48, 100))]
assert cols, "no tooltip under the bar"
left, right = min(cols), max(cols)
if int(sys.argv[3]) < 640:
    assert left >= 12, f"the tooltip starts at x {left}, left of the island at 12"
else:
    assert right <= 1267, f"the tooltip ends at x {right}, right of the island at 1267"
PY
done
ok "the tooltips of Apps and Keys stay inside the islands"

grep -q '"today"' "$flavor/waybar/config.jsonc" || fail "the month of the clock does not mark today"
ok "the month of the clock marks today"
