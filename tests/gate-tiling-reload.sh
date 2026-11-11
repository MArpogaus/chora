#!/bin/bash
# A new theme reaches the bar and the wallpaper (tiling-v2 review r2, 11),
# also when no unit runs them: after `theme: paper` and settings.reload the
# bar and the wallpaper of a session without units turn to Paper within 5 s.
# The bar runs through waybar.sh and the wallpaper through wallpaper.sh,
# as tests/persona/session.sh starts them.
#
# Usage: tests/gate-tiling-reload.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need swaybg
need jq
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling
fake=$(mktemp -d /tmp/gate-reload.XXXXXX)
tmp_dirs+=("$fake")
printf '#!/bin/sh\nexit 1\n' >"$fake/busctl"
chmod +x "$fake/busctl"
export PATH=$fake:$bin:$PATH
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
export CHORA_FLAVOR_DIR=$flavor
setsid dbus-run-session -- sh "$flavor/wallpaper.sh" >/dev/null 2>&1 &
run_pid $!
setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
run_pid $!
grant screen.capture
shot=$fake/reload.png
snap() { ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1; }
reads() {
    python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
bar, wall = px(60, 10), px(4, 400)
name = lambda c: "paper" if c[0] > 200 else "ink" if 8 <= c[0] <= 40 else "none"
print(name(bar), name(wall))
PY
}
for _ in $(seq 40); do
    sleep 0.5
    snap || continue
    [ "$(reads)" = "ink ink" ] && break
done
[ "$(reads)" = "ink ink" ] || fail "the bar and the wallpaper do not start in Ink: $(reads)"
ok "bar and wallpaper start in Ink"

mkdir -p "$XDG_CONFIG_HOME/chora"
"$bin/chora-tiling" --print-config >"$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
sed -i 's/^  theme: system/  theme: paper/' "$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"
got=
for _ in $(seq 10); do
    sleep 0.5
    snap || continue
    got=$(reads)
    [ "$got" = "paper paper" ] && break
done
[ "$got" = "paper paper" ] || fail "after the reload the bar and the wallpaper are: $got (bar wallpaper)"
ok "a theme reload turns the bar and the wallpaper to Paper without units"
