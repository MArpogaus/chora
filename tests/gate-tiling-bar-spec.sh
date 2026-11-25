#!/bin/bash
# The bar of the tiling flavor to the spec of design/looks/tiling-v2.md
# "Bar" (ti-21, the Hyprland-class look), in Ink and Paper, at the scale 1
# and 2 on the flat black of the headless output (so that the opacity of
# the surface shows). One cell per line of the spec, each with its own
# message:
#   bar-height    32 px high from the top edge of the island to its bottom
#   bar-top       8 px from the top edge of the output
#   bar-sides     12 px from the left and from the right edge
#   bar-radius    corner radius 10 (a corner of 12 or 8 fails at scale 2)
#   bar-surface   the island is `surface` at 92 %: 0.92 surface + 0.08 black
#   bar-hairline  a 1 px `line` hairline around the island, no gradient edge
#   bar-font      Inter 14 px weight 500: the width and the ink of Apps and
#                 Keys, the width of the clock (Inter, not the mono face; the bound follows
#                 the clock text, tests/clock-width.py)
#   bar-words     the words Apps and Keys and the power icon are visible
# Each cell is a measurement of the capture, no retry beyond the wait for
# the bar.
#
# Usage: tests/gate-tiling-bar-spec.sh   BIN: directory with the binaries
#   THEMES  the themes (default "ink paper")
#   SIZES   the sizes (default "1280x800 2560x1600@2")
#   SHOTS   a directory that keeps the capture of each round
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling
shots=${SHOTS:-}
red=0
[ -z "$shots" ] || mkdir -p "$shots"

palette() {
    case $1 in
    ink) echo "26 29 39 46 51 68 230 232 239" ;;
    paper) echo "255 255 255 213 216 225 27 30 39" ;;
    esac
}

round() {
    local theme=$1 size=$2 scale=1
    [[ $size == *@* ]] && scale=${size##*@}
    local tag=$theme-$size
    "$bin/chora-tiling" --print-config >"$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
    sed -i "s/^  theme: system/  theme: $theme/" "$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
    env_file=$XDG_CONFIG_HOME/$tag.env session_start "$size" -- "$bin/chora-tiling" --no-companions
    grant screen.capture
    PATH=$bin:$PATH CHORA_FLAVOR_DIR=$flavor setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >"$XDG_CONFIG_HOME/wb-$tag.log" 2>&1 &
    run_pid $!
    local shot=$XDG_CONFIG_HOME/$tag.png
    for _ in $(seq 40); do
        sleep 1
        ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || continue
        [ "$(pixel "$shot" $((640 * scale)) $((24 * scale)))" != "$(pixel "$shot" $((640 * scale)) $((300 * scale)))" ] && break
    done
    sleep 2
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || fail "$tag: the capture"
    [ -z "$shots" ] || cp "$shot" "$shots/bar-$tag.png"
    local now clock_bound
    now=$(date +%s)
    clock_bound=$(python3 -I "$repo/tests/clock-width.py" $((now - 60)) "$now") || fail "$tag: the clock bound"
    # shellcheck disable=SC2046
    python3 -I - "$repo/tests/png-pixel.py" "$shot" "$scale" "$tag" "$clock_bound" $(palette "$theme") <<'PY' || red=1
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = list(mod.rows(open(sys.argv[2], "rb").read()))
s, tag = int(sys.argv[3]), sys.argv[4]
clock_bound = tuple(float(v) for v in sys.argv[5].split())
surface, line, text = (tuple(int(v) for v in sys.argv[6 + 3 * i:9 + 3 * i]) for i in range(3))
W, H = len(rows[0]) // 4, len(rows)

def px(x, y):
    return tuple(rows[y][x * 4:x * 4 + 3])

failed = []

def die(cell, why):
    print(f"FAIL: {tag} {cell}: {why}", file=sys.stderr)
    failed.append(cell)

def near(a, b, t):
    return all(abs(x - y) <= t for x, y in zip(a, b))

bg = px(640 * s, 300 * s)
assert bg == (0, 0, 0), f"{tag}: the flat background is {bg}"
def bar(x, y):
    return px(x, y) != bg

y_mid = 24 * s
left = next((x for x in range(W) if bar(x, y_mid)), None)
right = next((x for x in range(W - 1, -1, -1) if bar(x, y_mid)), None)
if left != 12 * s:
    die("bar-sides", f"the left island starts at x {left}, not {12 * s} (12 px)")
if right != W - 12 * s - 1:
    die("bar-sides", f"the right island ends at x {right}, not {W - 12 * s - 1} (12 px from the edge {W})")
col = 70 * s
top = next((y for y in range(H) if bar(col, y)), None)
bottom = next((y for y in range(H - 1, -1, -1) if bar(col, y) and y < 100 * s), None)
if top != 8 * s:
    die("bar-top", f"the island starts at y {top}, not {8 * s} (8 px)")
if bottom - top + 1 != 32 * s:
    die("bar-height", f"the island is {bottom - top + 1} device px high, not {32 * s} (32 px)")

if s == 2:
    x0, y0, y1 = 12 * s, 8 * s, 40 * s - 1
    for name, dx, dy, y in (("top left", 1, 1, y0), ("bottom left", 1, -1, y1)):
        if bar(x0 + 4, y + dy * 4):
            die("bar-radius", f"the {name} corner of the island is not cut at 4 device px, the radius is below 10")
        if not bar(x0 + 6, y + dy * 6):
            die("bar-radius", f"the {name} corner of the island is cut at 6 device px, the radius is above 10")
else:
    if bar(12, 8) or bar(12, 39) or bar(13, 8):
        die("bar-radius", "the island has no rounded corner at 1x")

inside = px(14 * s + 1, 24 * s)
want = tuple(round(0.92 * c) for c in surface)
if not near(inside, want, 1):
    die("bar-surface", f"the island is {inside}, not surface {surface} at 92 % = {want}")

for name, (x, y), (xi, yi) in (("top", (col, top), (col, top + s)),
                               ("left", (left, y_mid), (left + s, y_mid)),
                               ("bottom", (col, bottom), (col, bottom - s))):
    if not near(px(x, y), line, 2):
        die("bar-hairline", f"the {name} edge is {px(x, y)}, not the line colour {line}")
    if near(px(xi, yi), line, 2) or not near(px(xi, yi), want, 1):
        die("bar-hairline", f"the {name} hairline is wider than 1 px or glows: {px(xi, yi)} inside it")

def ink_box(x0, x1, y0, y1):
    """Bounding box (css px) and coverage (css px squared) of the glyphs."""
    counts = {}
    for y in range(y0 * s, y1 * s):
        for x in range(x0 * s, x1 * s):
            counts[px(x, y)] = counts.get(px(x, y), 0) + 1
    ref = max(counts, key=counts.get)
    c0 = 0.2126 * ref[0] + 0.7152 * ref[1] + 0.0722 * ref[2]
    ct = 0.2126 * text[0] + 0.7152 * text[1] + 0.0722 * text[2]
    xs, cov = [], 0.0
    for y in range(y0 * s, y1 * s):
        for x in range(x0 * s, x1 * s):
            p = px(x, y)
            lum = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
            k = min(1.0, abs(lum - c0) / abs(ct - c0))
            if k > 0.3:
                xs.append(x)
            cov += k
    if not xs:
        return None
    return (max(xs) - min(xs) + 1) / s, cov / (s * s)

apps = ink_box(45, 92, 14, 34)
keys = ink_box(1190, 1232, 14, 34)
power = ink_box(1236, 1262, 14, 34)
if apps is None or apps[1] < 20:
    die("bar-words", "the word Apps is not in the bar")
if keys is None or keys[1] < 20:
    die("bar-words", "the word Keys is not in the bar")
if power is None or power[1] < 8:
    die("bar-words", "the power icon is not in the bar")
for word, found, (w_lo, w_hi, i_lo, i_hi) in (("Apps", apps, (32, 36, 122, 144)),
                                               ("Keys", keys, (30, 33, 112, 130))):
    if found is None:
        continue
    width, ink = found
    if not (w_lo <= width <= w_hi and i_lo <= ink <= i_hi):
        die("bar-font", f"{word} is {width:.1f} wide with {ink:.0f} of ink, not Inter 14 px weight 500 "
            f"({w_lo}-{w_hi} wide, {i_lo}-{i_hi} of ink)")
c_left = next(x for x in range(545 * s, 640 * s) if bar(x, y_mid))
c_right = next(x for x in range(760 * s, 640 * s, -1) if bar(x, y_mid))
clock = ink_box(c_left // s + 4, c_right // s - 3, 14, 34)
if clock is None:
    die("bar-words", "no clock in the bar")
else:
    c_lo, c_hi = clock_bound
    if not c_lo <= clock[0] <= c_hi:
        die("bar-font", f"the clock is {clock[0]:.1f} wide, not Inter 14 px weight 500 ({c_lo:.0f}-{c_hi:.0f} for its text)")
if failed:
    sys.exit(1)
print(f"ok: {tag}: island {left},{top} to {right},{bottom}, surface {inside}, hairline, Apps {apps}, Keys {keys}, clock {clock[0]:.1f}")
PY
    kill "${pids[@]}" "${run_pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
    run_pids=()
}

temp_config
mkdir -p "$XDG_CONFIG_HOME/chora"
for theme in ${THEMES:-ink paper}; do
    for size in ${SIZES:-1280x800 2560x1600@2}; do
        round "$theme" "$size"
    done
done
[ "$red" = 0 ] || fail "the bar differs from the spec of tiling-v2 (see the lines above)"
ok "the bar is 32 high, 8 from the top, 12 from the sides, radius 10, surface at 92 %, a 1 px line hairline, Inter 14 px weight 500, with Apps, Keys and the power icon"
