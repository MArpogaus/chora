#!/bin/bash
# The Run box of the xp flavor (xp-2): Super+Space gives fuzzel the
# installed programs as rows, sorted by name, and text that finds nothing
# shows the list again under the row "Nothing found for ...", so Enter
# never closes the box without a word. First a stand-in for fuzzel keeps
# the rows that it gets and answers text that matches nothing. Then the
# real fuzzel runs: after text without a match and Enter the box still
# shows, in the face colour of XP, and Escape closes it. The box is a
# layer surface, which no query lists: it shows when the middle of a
# capture has that colour.
#
# Usage: tests/gate-xp-run.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the capture xp-run-nothing.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need fuzzel
need_bins chora choractl chora-xp
temp_config
work=$XDG_CONFIG_HOME
export HOME=$work/home
export XDG_DATA_HOME=$work/data
export XDG_DATA_DIRS=$work/none
mkdir -p "$HOME" "$XDG_DATA_HOME/applications" "$work/fake"
entry() {
    printf '[Desktop Entry]\nType=Application\nName=%s\nExec=%s\n' "$2" "$3" >"$XDG_DATA_HOME/applications/$1.desktop"
}
entry zebra Zebra "foot -a zebra sh -c 'sleep 600'"
entry alpha alpha "foot -a alpha sh -c 'sleep 600'"
entry ghost Ghost "chora-no-such-program --flag"
cat >"$work/fake/fuzzel" <<FAKE
#!/bin/sh
[ ! -e "$work/fake/real" ] || exec /usr/bin/env PATH="$bin:$PATH" fuzzel "\$@"
n=\$((\$(cat "$work/fake/calls" 2>/dev/null || echo 0) + 1))
echo "\$n" >"$work/fake/calls"
case " \$* " in *" --dmenu "*) cat >"$work/fake/rows.\$n" ;; esac
[ "\$n" != 1 ] || echo zzqq
FAKE
chmod +x "$work/fake/fuzzel"
export PATH=$work/fake:$bin:$PATH
session_start 1280x800 -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
grant screen.capture input.inject
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }

key Super+space
for _ in $(seq 40); do
    [ -e "$work/fake/calls" ] && break
    sleep 0.25
done
[ -e "$work/fake/calls" ] || fail "Super+Space did not run the launcher"
sleep 2
rows=$work/fake/rows.1
[ -s "$rows" ] || fail "the Run box got no rows: fuzzel lists the programs itself, and Enter on text without a match closes it without a word"
line_of() { grep -nxF "$1" "$rows" | head -n 1 | cut -d: -f1; }
a=$(line_of alpha) z=$(line_of Zebra)
if [ -z "$a" ] || [ -z "$z" ] || [ "$a" -ge "$z" ]; then fail "alpha and Zebra are not rows sorted by name: $(tr '\n' '|' <"$rows")"; fi
grep -qxF Ghost "$rows" && fail "Ghost, whose program is not installed, is a row"
ok "the Run box lists the installed programs, sorted by name"
[ "$(head -n 1 "$work/fake/rows.2" 2>/dev/null)" = "Nothing found for \"zzqq\"" ] ||
    fail "text that finds nothing gives no row that says so: $(head -n 1 "$work/fake/rows.2" 2>/dev/null)"
ok "text that finds nothing shows the list again under the row Nothing found"

: >"$work/fake/real"
shot=$work/run.png
shown() {
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || return 1
    python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
hits = sum(px(x, y) == (0xec, 0xe9, 0xd8) for x in range(500, 780, 4) for y in range(300, 500, 4))
sys.exit(0 if hits > 100 else 1)
PY
}
wait_shown() {
    for _ in $(seq 40); do
        shown && return 0
        sleep 0.25
    done
    return 1
}
key Super+space
wait_shown || fail "the Run box does not show"
sleep 1
echo '[{"text": {"text": "zzqq"}}]' | ctl inject >/dev/null || fail "inject text"
sleep 0.5
key Return
sleep 2
shown || fail "the Run box closed without a word after text that finds nothing"
[ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/xp-run-nothing.png"
key Escape
for _ in $(seq 20); do
    shown || break
    sleep 0.25
done
shown && fail "Escape keeps the Run box"
ok "after text that finds nothing the Run box shows again, and Escape closes it"
