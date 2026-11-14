#!/bin/bash
# The Apps list of the tiling flavor (ti-2): chora-tiling --launch fuzzel
# gives fuzzel the installed programs as rows, sorted by name, so the
# Screenshot of the flavor is not the first row and Enter alone takes no
# screenshot; a program that is not installed is no row; text that finds
# nothing shows the list again under the row "Nothing found for ...".
# First a stand-in for fuzzel keeps the rows that it gets and answers
# text that matches nothing. Then the real fuzzel runs: Enter starts the
# first program of the list, and after text without a match and Enter the
# launcher still shows. The launcher is a layer surface, which no query
# lists: it shows when the middle of a capture has the surface colour of
# Ink. fuzzel before 1.11 does not know selection-radius of the theme,
# which the plugin may write again at any time, so the gate removes the
# line before each start.
#
# Usage: tests/gate-tiling-apps.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the capture tiling-apps-nothing.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need fuzzel
need_bins chora choractl chora-tiling
temp_config
work=$XDG_CONFIG_HOME
export HOME=$work/home
export XDG_DATA_HOME=$work/data
export XDG_DATA_DIRS=$work/none
export XDG_PICTURES_DIR=$HOME/Pictures
export CHORA_FLAVOR_DIR=$repo/flavors/tiling
mkdir -p "$HOME" "$XDG_DATA_HOME/applications" "$work/fake"
entry() {
    printf '[Desktop Entry]\nType=Application\nName=%s\nExec=%s\n' "$2" "$3" >"$XDG_DATA_HOME/applications/$1.desktop"
}
entry zebra Zebra "foot -a zebra sh -c 'sleep 600'"
entry alpha alpha "foot -a alpha sh -c 'sleep 600'"
entry ghost Ghost "chora-no-such-program --flag"
cat >"$work/fake/fuzzel" <<FAKE
#!/bin/sh
n=\$((\$(cat "$work/fake/calls" 2>/dev/null || echo 0) + 1))
echo "\$n" >"$work/fake/calls"
case " \$* " in *" --dmenu "*) cat >"$work/fake/rows.\$n" ;; esac
[ "\$n" != 1 ] || echo zzqq
FAKE
chmod +x "$work/fake/fuzzel"
real_path=$bin:$PATH
export PATH=$real_path
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture input.inject

PATH=$work/fake:$real_path timeout 20 "$bin/chora-tiling" --launch fuzzel >/dev/null 2>&1
rows=$work/fake/rows.1
[ -s "$rows" ] || fail "the launcher got no rows: fuzzel lists and sorts the programs itself"
line_of() { grep -nxF "$1" "$rows" | head -n 1 | cut -d: -f1; }
a=$(line_of alpha) s=$(line_of Screenshot) z=$(line_of Zebra)
if [ -z "$a" ] || [ -z "$s" ] || [ -z "$z" ]; then fail "alpha, Screenshot or Zebra is no row: $(tr '\n' '|' <"$rows")"; fi
if [ "$a" -ge "$s" ] || [ "$s" -ge "$z" ]; then fail "the rows are not sorted by name: $(tr '\n' '|' <"$rows")"; fi
[ "$(head -n 1 "$rows")" != Screenshot ] || fail "Screenshot is the first row"
grep -qxF Ghost "$rows" && fail "Ghost, whose program is not installed, is a row"
ok "the rows are sorted by name, Screenshot is not the first, and a program that is not installed is no row"
[ "$(head -n 1 "$work/fake/rows.2" 2>/dev/null)" = "Nothing found for \"zzqq\"" ] ||
    fail "text that finds nothing gives no row that says so: $(head -n 1 "$work/fake/rows.2" 2>/dev/null)"
[ "$(sed -n 2p "$work/fake/rows.2")" = "$(head -n 1 "$rows")" ] || fail "the list does not show again under the row"
ok "text that finds nothing shows the list again under the row Nothing found"

shot=$work/apps.png
shown() {
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || return 1
    python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
hits = sum(px(x, y) == (0x1a, 0x1d, 0x27) for x in range(500, 780, 4) for y in range(300, 500, 4))
sys.exit(0 if hits > 100 else 1)
PY
}
fuzzel_version=$(fuzzel --version 2>/dev/null | sed -n 's/^fuzzel version: \([0-9.]*\).*/\1/p')
launcher() {
    if [ "$(printf '%s\n1.11\n' "${fuzzel_version:-0}" | sort -V | head -n 1)" != 1.11 ]; then
        sed -i '/^selection-radius=/d' "$XDG_RUNTIME_DIR/chora-tiling/theme/fuzzel.ini"
    fi
    started=$(ctl run "$bin/chora-tiling" --launch fuzzel) || fail "the run of the launcher"
    run_pid "$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")"
    for _ in $(seq 40); do
        shown && return 0
        sleep 0.25
    done
    fail "the launcher does not show"
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }

launcher
sleep 1
key Return
ctl wait --timeout 15 'app_id:alpha kind:surface' >/dev/null || fail "Enter in the new list did not start alpha, its first row"
sleep 1
[ -z "$(find "$XDG_PICTURES_DIR" -name '*.png' 2>/dev/null)" ] || fail "Enter in the new list took a screenshot"
ok "Enter in the new list starts the first program by name, and takes no screenshot"

launcher
sleep 1
echo '[{"text": {"text": "zzqq"}}]' | ctl inject >/dev/null || fail "inject text"
sleep 0.5
key Return
sleep 2
shown || fail "the launcher closed without a word after text that finds nothing"
[ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-apps-nothing.png"
key Escape
for _ in $(seq 20); do
    shown || break
    sleep 0.25
done
shown && fail "Escape keeps the launcher"
ok "after text that finds nothing the launcher shows again, and Escape closes it"
