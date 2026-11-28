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
# Ink.
#
# The helper is findable from the launcher (ti-50): one row with the word
# Helper and the name of the helper, the row of Super+a (chora-chat). A
# stand-in for chora-chat --door says "atlas · idle" or "No helper" (the file
# door holds the word).
#
# The helper stands first (ti-50, rpt1-6): the nine rows that fuzzel shows
# always hold it, and Enter on the preselected row opens a chat, which Escape
# closes. The programs follow, and the rows of the flavor (its theme,
# settings and move rows, whose program is chora-tiling) stand last. Look
# means one of the four looks (ti-55), so the rows of the colours say theme.
#
# The rows that move a window stand in the list only with a window that has
# the focus, as the Looks menu has them (rtmr1-3): the list of a desktop with
# no window has none, the list with a window has all three.
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
case " \$* " in *" --dmenu "*) sed 's/\\x00icon\\x1f.*\$//' >"$work/fake/rows.\$n" ;; esac
[ "\$n" != 1 ] || echo zzqq
FAKE
chmod +x "$work/fake/fuzzel"
printf '#!/bin/sh\ncat "%s/fake/door"\n' "$work" >"$work/fake/chora-chat"
chmod +x "$work/fake/chora-chat"
echo "atlas · idle" >"$work/fake/door"
real_path=$bin:$PATH
export PATH=$real_path
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture input.inject

mkdir -p "$work/fake-bare"
cat >"$work/fake-bare/fuzzel" <<FAKE
#!/bin/sh
case " \$* " in *" --dmenu "*) sed 's/\\x00icon\\x1f.*\$//' >"$work/bare-rows" ;; esac
FAKE
chmod +x "$work/fake-bare/fuzzel"
PATH=$work/fake-bare:$real_path timeout 20 "$bin/chora-tiling" --launch fuzzel >/dev/null 2>&1
[ -s "$work/bare-rows" ] || fail "the launcher of a desktop with no window got no rows"
grep -q '^Move window to ' "$work/bare-rows" && fail "a desktop with no window offers to move one (rtmr1-3): $(tr '\n' '|' <"$work/bare-rows")"
grep -qxF "Dark theme" "$work/bare-rows" || fail "the rows of the theme are gone with no window: $(tr '\n' '|' <"$work/bare-rows")"
ok "a desktop with no window has no row that moves a window, and the other rows of the flavor stay (rtmr1-3)"
ctl run foot -a mover sh -c 'sleep 600' >/dev/null || fail "the run of the window"
ctl wait --timeout 15 'app_id:mover kind:surface' >/dev/null || fail "the window did not show"
bare_rows_move() {
    PATH=$work/fake-bare:$real_path timeout 20 "$bin/chora-tiling" --launch fuzzel >/dev/null 2>&1
    grep -q "^Move window to " "$work/bare-rows"
}
until_ok bare_rows_move 40 ||
    fail "a desktop with a focused window offers no row that moves it (rtmr1-3): $(tr '\n' '|' <"$work/bare-rows")"
ok "a desktop with a window that has the focus offers the rows that move it (rtmr1-3)"

PATH=$work/fake:$real_path timeout 20 "$bin/chora-tiling" --launch fuzzel >/dev/null 2>&1
rows=$work/fake/rows.1
[ -s "$rows" ] || fail "the launcher got no rows: fuzzel lists and sorts the programs itself"
line_of() { grep -nxF "$1" "$rows" | head -n 1 | cut -d: -f1; }
a=$(line_of alpha) s=$(line_of Screenshot) z=$(line_of Zebra)
if [ -z "$a" ] || [ -z "$s" ] || [ -z "$z" ]; then fail "alpha, Screenshot or Zebra is no row: $(tr '\n' '|' <"$rows")"; fi
if [ "$a" -ge "$s" ] || [ "$s" -ge "$z" ]; then fail "the rows are not sorted by name: $(tr '\n' '|' <"$rows")"; fi
[ "$(head -n 1 "$rows")" != Screenshot ] || fail "Screenshot is the first row"
grep -qxF Ghost "$rows" && fail "Ghost, whose program is not installed, is a row"
for row in "Move window to the xp desktop" "Move window to the media center desktop" "Move window to the construct desktop"; do
    grep -qxF "$row" "$rows" || fail "no launcher row \"$row\": $(tr '\n' '|' <"$rows")"
done
grep -qxF "Move window to the tiling desktop" "$rows" && fail "tiling is offered to move a window to itself"
ok "the rows are sorted by name, Screenshot is not the first, and a program that is not installed is no row"
ok "the launcher moves a window to the xp, media center and construct desktop in the words of the key list"
[ "$(head -n 1 "$work/fake/rows.2" 2>/dev/null)" = "Nothing found for \"zzqq\"" ] ||
    fail "text that finds nothing gives no row that says so: $(head -n 1 "$work/fake/rows.2" 2>/dev/null)"
[ "$(sed -n 2p "$work/fake/rows.2")" = "$(head -n 1 "$rows")" ] || fail "the list does not show again under the row"
ok "text that finds nothing shows the list again under the row Nothing found"

grep -qxF "Chat with a helper (atlas)" "$rows" ||
    fail "no launcher row \"Chat with a helper (atlas)\" for the helper that the door names: $(tr '\n' '|' <"$rows")"
[ "$(grep -ci 'helper' "$rows")" = 1 ] || fail "more than one launcher row for the helper: $(tr '\n' '|' <"$rows")"
[ "$(head -n 1 "$rows")" = "Chat with a helper (atlas)" ] ||
    fail "the first row is not the helper row \"Chat with a helper (atlas)\": $(tr '\n' '|' <"$rows")"
last_program=0 first_flavor=999
n=0
while IFS= read -r row; do
    n=$((n + 1))
    case $row in
    "Dark theme" | "Light theme" | "System theme" | "Edit settings" | "Move window to "*) [ "$first_flavor" -le "$n" ] || first_flavor=$n ;;
    "Chat with a helper"*) ;;
    *) last_program=$n ;;
    esac
done <"$rows"
[ "$last_program" -lt "$first_flavor" ] || fail "a flavor row stands before a program: $(tr '\n' '|' <"$rows")"
if grep -iwq look "$rows"; then fail "a row of the flavor has the word look: $(tr '\n' '|' <"$rows")"; fi
for row in "Dark theme" "Light theme" "System theme"; do
    grep -qxF "$row" "$rows" || fail "no launcher row \"$row\": $(tr '\n' '|' <"$rows")"
done
ok "the helper row stands first, the rows of the flavor last, and none says look"
echo "No helper" >"$work/fake/door"
PATH=$work/fake:$real_path timeout 20 "$bin/chora-tiling" --launch fuzzel >/dev/null 2>&1
grep -qxF "Chat with a helper" "$work/fake/rows.3" ||
    fail "without a helper the row is not plain \"Chat with a helper\": $(tr '\n' '|' <"$work/fake/rows.3")"
ok "the launcher has one helper row, with the name of the helper (ti-50)"

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
launcher() {
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
key Down
sleep 0.5
key Return
ctl wait --timeout 15 'app_id:alpha kind:surface' >/dev/null || fail "Down and Enter in the new list did not start alpha, its first program"
sleep 1
[ -z "$(find "$XDG_PICTURES_DIR" -name '*.png' 2>/dev/null)" ] || fail "Enter in the new list took a screenshot"
ok "Down and Enter in the new list start the first program by name, and take no screenshot"

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
