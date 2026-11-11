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
shot=$XDG_CONFIG_HOME/programs.png
snap() {
    sleep 3
    ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
}
start() {
    pid=$(ctl run "$@" | sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p')
    [ -n "$pid" ] || fail "the run of $1"
    run_pid "$pid"
}

start chora-tiling --launch foot sh -c 'printf "\033[31mwarning: red text\033[0m\n"; sleep 600'
for _ in $(seq 60); do
    ctl query 'app_id:foot kind:surface' | grep -q '"owner": *"tiling"' && break
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
