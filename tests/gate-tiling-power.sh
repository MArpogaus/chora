#!/bin/bash
# The power and screenshot doors of tiling (tiling-v2 review r2, 1 and 2),
# in a session where chora-tiling has no CHORA_FLAVOR_DIR, as in a test
# session: Super+Shift+e opens the power menu within 2 s, three clicks on
# the power icon open it three times, Super+Print writes a file and says so
# in a notice, the Apps list has an entry "Screenshot" that takes one, and
# a missing script is an error on stderr and a notice, never a silent
# `true`. Needs fuzzel, grim, waybar and jq; notify-send is a stand-in.
#
# Usage: tests/gate-tiling-power.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need fuzzel
need grim
need jq
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling
fake=$(mktemp -d /tmp/gate-power.XXXXXX)
tmp_dirs+=("$fake")
marks=$fake/marks
: >"$marks"
printf '#!/bin/sh\necho notify "$@" >>"%s"\n' "$marks" >"$fake/notify-send"
printf '#!/bin/sh\necho loginctl "$@" >>"%s"\n' "$marks" >"$fake/loginctl"
chmod +x "$fake/notify-send" "$fake/loginctl"
export PATH=$fake:$bin:$PATH
export HOME=$fake/home
export XDG_PICTURES_DIR=$fake/pictures
mkdir -p "$HOME"
unset CHORA_FLAVOR_DIR
temp_config
tmp_dirs+=("$fake")
session_start 1280x800 -- "$bin/chora-tiling"
CHORA_FLAVOR_DIR=$flavor setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
run_pid $!
grant screen.capture input.inject
shot=$fake/bar.png
for _ in $(seq 40); do
    sleep 0.5
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || continue
    [ "$(pixel "$shot" 640 24)" != "$(pixel "$shot" 640 300)" ] && break
done
keys() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
click() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null || fail "the click at $1"; }
menu_open() {
    ctl capture --output HEADLESS-1 "$fake/menu.png" >/dev/null 2>&1 || return 1
    python3 -I - "$repo/tests/png-pixel.py" "$fake/menu.png" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
hits = sum(px(x, y) == (0x1a, 0x1d, 0x27) for x in range(580, 700, 4) for y in range(340, 460, 4))
sys.exit(0 if hits > 200 else 1)
PY
}
wait_menu() {
    for _ in $(seq 8); do
        sleep 0.25
        menu_open && return 0
    done
    return 1
}
close_menu() {
    keys Escape
    for _ in $(seq 20); do
        menu_open || return 0
        sleep 0.25
    done
    fail "the menu did not close"
}

keys Super+Shift+e
wait_menu || fail "Super+Shift+e opened no power menu within 2 s"
close_menu
ok "Super+Shift+e opens the power menu within 2 s"

for n in 1 2 3; do
    click 1238 24
    wait_menu || fail "click $n of 3 on the power icon opened no menu"
    close_menu
done
ok "the power icon opens the menu 3 of 3 times"

keys Super+Print
for _ in $(seq 20); do
    ls "$XDG_PICTURES_DIR"/Screenshots/*.png >/dev/null 2>&1 && break
    sleep 0.5
done
file=
for f in "$XDG_PICTURES_DIR"/Screenshots/*.png; do
    file=$f
    break
done
[ -s "$file" ] || fail "Super+Print wrote no file"
sleep 1
grep -q '^notify.*Screenshot saved' "$marks" || fail "Super+Print showed no notice"
ok "Super+Print writes $(basename "$file") and notifies"

: >"$marks"
if [ -f "$flavor/applications/chora-screenshot.desktop" ]; then
    grep -q '^Name=Screenshot$' "$flavor/applications/chora-screenshot.desktop" || fail "the entry is not named Screenshot"
else
    fail "no Apps entry Screenshot in the flavor"
fi
rm -f "$XDG_PICTURES_DIR"/Screenshots/*.png
keys Super+space
wait_menu || fail "Super+space opened no launcher"
sleep 0.5
echo '[{"text": {"text": "Screenshot"}}]' | ctl inject - >/dev/null || fail "typing Screenshot"
sleep 1
keys Return
for _ in $(seq 20); do
    ls "$XDG_PICTURES_DIR"/Screenshots/*.png >/dev/null 2>&1 && break
    sleep 0.5
done
ls "$XDG_PICTURES_DIR"/Screenshots/*.png >/dev/null 2>&1 || fail "the Apps entry Screenshot took no screenshot"
ok "Apps lists Screenshot, and it takes one"

lost=$(mktemp -d "$fake/lost.XXXXXX")
cp "$bin/chora-tiling" "$lost/chora-tiling"
out=$(env -u CHORA_FLAVOR_DIR HOME="$lost" XDG_CONFIG_HOME="$lost" XDG_RUNTIME_DIR="$lost" \
    "$lost/chora-tiling" --launch power 2>&1)
status=$?
[ "$status" != 0 ] || fail "a missing power.sh exited 0"
grep -q 'power' <<<"$out" || fail "a missing power.sh said nothing on stderr: $out"
grep -q '^notify.*Power menu' "$marks" || fail "a missing power.sh showed no notice"
ok "a missing script is an error and a notice"

stub=$(mktemp -d "$fake/stub.XXXXXX")
printf '#!/bin/sh\necho "$@" >"%s/args"\ncat >/dev/null\n' "$stub" >"$stub/fuzzel"
chmod +x "$stub/fuzzel"
PATH=$stub:$PATH sh "$flavor/power.sh" >/dev/null 2>&1
grep -q -- '--prompt Power' "$stub/args" || fail "the power menu has no prompt: $(cat "$stub/args" 2>/dev/null)"
LANG=de_DE.UTF-8 PATH=$stub:$PATH sh "$flavor/power.sh" >/dev/null 2>&1
grep -q -- '--prompt Ausschalten' "$stub/args" || fail "the German power menu has no prompt: $(cat "$stub/args" 2>/dev/null)"
ok "the power menu shows a prompt, in German too"
