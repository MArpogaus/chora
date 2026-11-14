#!/bin/bash
# The programs of the tiling flavor in the palette (tiling-v2 review 5, 6
# and 8): the terminal has its text 16 px inside the window on the body of
# Ink, not at the rounded corner, and the launcher is opaque: a window
# behind it does not show through. Both start the way the bindings start
# them, with chora-tiling --launch, which finds the files that the plugin
# wrote for the palette.
#
# Usage: tests/gate-tiling-programs.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need fuzzel
need_bins chora choractl chora-tiling chora-probe
export PATH=$bin:$PATH
export CHORA_FLAVOR_DIR=$repo/flavors/tiling
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture
for file in docs/configuration.md dist/arch/PKGBUILD crates/chora/Cargo.toml; do
    grep -q "fuzzel.*1\.11" "$repo/$file" || fail "$file does not name fuzzel 1.11, the first with mouse support"
done
ok "the docs and the packages name fuzzel 1.11 as the least version for the mouse"
shot=$XDG_CONFIG_HOME/programs.png
snap() {
    sleep 3
    ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
}
start() {
    started=$(ctl run "$@") || fail "the run of $1"
    pid=$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")
    [ -n "$pid" ] || fail "the run of $1"
    run_pid "$pid"
}

start chora-tiling --launch swaybg
sleep 3
snap
python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY' || fail "the wallpaper is not Ink"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
edge, glow, far = px(0, 400), px(4, 4), px(1275, 795)
assert all(abs(a - b) <= 4 for a, b in zip(edge, (0x10, 0x11, 0x19))), f"the wallpaper at 0,400 is {edge}, not Ink base"
assert glow[2] > 0x40 and glow[0] > 0x25, f"no violet glow at the top left: {glow}"
assert far[1] > 0x22 and far[0] < 0x18, f"no teal glow at the bottom right: {far}"
PY
ok "the wallpaper is Ink with two corner glows"

start chora-tiling --launch foot sh -c 'printf "\033[31mwarning: red text\033[0m\n"; sleep 600'
for _ in $(seq 60); do
    seen=$(ctl query 'app_id:foot kind:surface') || { sleep 0.5; continue; }
    grep -q '"owner": *"tiling"' <<<"$seen" && break
    sleep 0.5
done
snap
python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY' || fail "the terminal is not in the palette or not padded"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
assert px(400, 400) == (0x16, 0x19, 0x22), f"the body is {px(400, 400)}, not 161922"
body = px(400, 400)
first = next(x for x in range(14, 400) if any(px(x, y) != body for y in range(30, 120)))
assert first - 14 >= 12, f"the first text pixel is {first - 14} px inside the window, not 12 or more"
assert any(px(x, y)[0] > 200 > px(x, y)[2] for x in range(first, first + 200) for y in range(30, 120)), "no red text in the palette"
PY
ok "the terminal has the palette and its text is padded away from the corners"

start chora-tiling --launch fuzzel
for _ in $(seq 40); do
    ctl query 'app_id:fuzzel' >/dev/null 2>&1 && break
    sleep 0.5
done
snap
python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY' || fail "the launcher is not opaque or not in the palette"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
want = (0x1a, 0x1d, 0x27)
hits = sum(px(x, y) == want for x in range(500, 780, 4) for y in range(300, 500, 4))
assert hits > 100, f"only {hits} of the samples are the surface colour: the panel is not opaque Ink"
PY
ok "the launcher is opaque and in the palette"

python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY' || fail "the selected row does not stand out from the panel"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
def lum(c):
    v = [x / 255 for x in c]
    v = [x / 12.92 if x <= 0.03928 else ((x + 0.055) / 1.055) ** 2.4 for x in v]
    return 0.2126 * v[0] + 0.7152 * v[1] + 0.0722 * v[2]
panel = (0x1a, 0x1d, 0x27)
rows_ = [(x, y) for x in range(420, 860, 4) for y in range(250, 560, 2) if px(x, y) not in (panel,) and abs(lum(px(x, y)) - lum(panel)) > 0.05]
bars = [(x, y) for x, y in rows_ if px(x, y) == (0xa7, 0x8b, 0xfa)]
assert len(bars) > 300, f"only {len(bars)} samples of the selection colour"
a, b = lum((0xa7, 0x8b, 0xfa)), lum(panel)
assert (max(a, b) + 0.05) / (min(a, b) + 0.05) >= 3.0
PY
ok "the selected row of the launcher stands out from the panel"
