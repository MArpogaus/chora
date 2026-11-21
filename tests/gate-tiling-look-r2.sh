#!/bin/bash
# Three honest states of the tiling flavor (ti-22, ti-23, ti-24), in Ink
# and Paper, on a session bus where no notification server was started:
#   notice   Super+Print says "Screenshot saved" with the path, or
#            "Screenshot failed" when grim fails: chora-notify, started in
#            the cell as its unit starts it, lists the notice (`state.get`
#            of notify), and it stands at the bottom right
#   tooltip  the tooltip of Apps leaves with the pointer, after a click
#            on Apps and Escape, and never stays over a client
#   door     the bar has the power icon and the lock key of the key list;
#            the bar word of the helper is the one door of the chat: no
#            "Chat ..." row floats over a tile, with a helper or without
# Each cell is exact: no tolerance, no retry beyond the wait for a state.
#
# Usage: tests/gate-tiling-look-r2.sh   BIN: directory with the binaries
#   THEMES  the themes (default "ink paper")
#   CELLS   a regular expression of the cells that run (default all)
#   SHOTS   a directory that keeps the capture of each cell
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling chora-assistant chora-chat
need foot
need fuzzel
need waybar
need grim
need gdbus
need jq
need mkfifo
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work/shots}
mkdir -p "$work/chora" "$work/data/chora/huds" "$shots" "$work/stub" "$work/home"
while read -r name dir; do
    cp -rL "$dir" "$work/data/chora/huds/$name"
done < <("$repo/tests/hud-folders.sh")
export PATH=$work/stub:$bin:$PATH
export CHORA_BIN=$bin
export HOME=$work/home
export XDG_PICTURES_DIR=$work/home/Pictures
export LC_ALL=C
flavor=$repo/flavors/tiling
bus=$(dbus-daemon --session --print-address --fork 2>/dev/null | head -n 1 | cut -d, -f1)
[ -n "$bus" ] || fail "no session bus"
export DBUS_SESSION_BUS_ADDRESS=$bus
export CHORA_TEST_KEEP_BUS=1
printf '#!/bin/sh\n[ -e "%s/stub/grim-fails" ] && { echo "compositor refused the capture" >&2; exit 1; }\nexec %s "$@"\n' "$work" "$(command -v grim)" >"$work/stub/grim"
chmod +x "$work/stub/grim"

cell_runs() { [[ $1 =~ ^(${CELLS:-.*})$ ]]; }
keys() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
move() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}]" | ctl inject - >/dev/null || fail "the move to $1,$2"; }
click() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null || fail "the click at $1,$2"; }
shoot() { ctl capture --output HEADLESS-1 "$1" >/dev/null 2>&1; }
differing() {
    python3 -I - "$repo/tests/png-pixel.py" "$@" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
a, b = (list(mod.rows(open(p, "rb").read())) for p in sys.argv[2:4])
x0, y0, x1, y1 = (int(v) for v in sys.argv[4:8])
print(sum(a[y][x * 4:x * 4 + 4] != b[y][x * 4:x * 4 + 4] for y in range(y0, y1) for x in range(x0, x1)))
PY
}
bar_shows() {
    local shot=$work/up.png
    for _ in $(seq 30); do
        shoot "$shot" && [ "$(pixel "$shot" 570 24)" != "$(pixel "$shot" 570 44)" ] && return 0
        sleep 0.5
    done
    return 1
}

start_round() {
    local theme=$1
    "$bin/chora-tiling" --print-config >"$work/chora/chora-tiling.yaml"
    sed -i "s/^  theme: system/  theme: $theme/" "$work/chora/chora-tiling.yaml"
    printf 'enabled: true\nposition: null\nhud: neutral\nhuds:\n- {when: {space: tiling}, folder: tiling}\n' >"$work/chora/chora-assistant.yaml"
    session_start 1280x800 -- "$bin/chora-tiling"
    grant screen.capture input.inject methods.call
    CHORA_FLAVOR_DIR=$flavor setsid sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >"$shots/waybar-$theme.log" 2>&1 &
    run_pid $!
    CHORA_FLAVOR_DIR=$flavor setsid sh "$flavor/wallpaper.sh" >/dev/null 2>&1 &
    run_pid $!
    rm -f "$work/voice.in"
    mkfifo "$work/voice.in"
    python3 -I "$repo/tests/fake-voice.py" "$work/claims" <"$work/voice.in" >"$shots/voice-$theme.log" 2>&1 &
    pids+=($!)
    exec 3>"$work/voice.in"
    XDG_DATA_HOME=$work/data "$bin/chora-assistant" >"$shots/assistant-$theme.log" 2>&1 &
    pids+=($!)
    for i in 1 2; do ctl run "$bin/chora-tiling" --launch foot -T "w$i" sh -c 'echo hello; sleep 600' >/dev/null || fail "$theme: foot"; done
    bar_shows || fail "$theme: no bar after 15 s: $(tail -n 3 "$shots/waybar-$theme.log")"
    pace 4
}

end_round() {
    exec 3>&-
    kill "${pids[@]}" "${run_pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
    run_pids=()
    rm -rf "$XDG_PICTURES_DIR"
}

cell_tooltip() {
    local theme=$1 base=$work/tip-base.png shot=$work/tip.png door name x y x0 x1 shown
    move 300 500
    settle
    shoot "$base" || fail "$theme tooltip: the capture"
    for door in "Apps 42 25 14 400" "Helper 955 24 700 1262" "Keys 1190 24 700 1262" "Power 1247 24 700 1262"; do
        read -r name x y x0 x1 <<<"$door"
        move 300 500
        sleep 0.5
        move "$x" "$y"
        sleep 0.3
        move $((x + 1)) $((y + 1))
        pace 3
        shoot "$shot" || fail "$theme tooltip: the capture"
        shown=$(differing "$base" "$shot" "$x0" 56 "$x1" 110)
        [ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-tooltip-$name-$theme.png"
        [ "$shown" = 0 ] || fail "$theme tooltip: the tooltip of $name stands over the tile while the pointer rests on it ($shown pixels differ)"
    done
    move 42 25
    sleep 0.3
    move 43 26
    settle
    keys Super+space
    sleep 1.5
    keys Escape
    pace 3
    shoot "$shot" || fail "$theme tooltip: the capture"
    shown=$(differing "$base" "$shot" 14 56 400 110)
    [ "$shown" = 0 ] || fail "$theme tooltip: something of the bar stands over the tile after the launcher closed ($shown pixels differ)"
    ok "$theme tooltip: no tooltip of a door of the bar stands over a tile"
}

cell_door() {
    local theme=$1 shot=$work/door.png rows word
    ctl run "$bin/chora-tiling" --launch foot -T w3 sh -c 'echo hello; sleep 600' >/dev/null || fail "$theme door: foot"
    settle
    echo 'voice.state {"state": "listening", "task": "", "said": ""}' >&3
    pace 3
    word=$(sh "$flavor/waybar/helper.sh" | jq -r .text)
    [ "$word" = "No helper" ] || fail "$theme door: the bar word is \"$word\" without a helper"
    shoot "$shot" || fail "$theme door: the capture"
    [ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-door-none-$theme.png"
    rows=$(ctl query '*' | jq -r '[.[] | select(.id | startswith("hud-row-")) | .id] | join(" ")')
    [ -z "$rows" ] || fail "$theme door: a \"Chat ...\" row floats over the tiles beside the bar word \"$word\": $rows"
    echo 'voice.state {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4}' >&3
    pace 3
    shoot "$shot" || fail "$theme door: the capture"
    [ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-door-working-$theme.png"
    rows=$(ctl query '*' | jq -r '[.[] | select(.id | startswith("hud-row-")) | .id] | join(" ")')
    [ -z "$rows" ] || fail "$theme door: a row of a helper floats over the tiles: $rows"
    ok "$theme door: the bar word is the one door, no \"Chat ...\" row floats over the tiles"
}

cell_power() {
    local theme=$1 shot=$work/power.png
    shoot "$shot" || fail "$theme power: the capture"
    [ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-bar-power-$theme.png"
    python3 -I - "$repo/tests/png-pixel.py" "$shot" <<'PY' || fail "$theme power: no power icon at the right end of the bar"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rows = list(mod.rows(open(sys.argv[2], "rb").read()))
px = lambda x, y: tuple(rows[y][x * 4:x * 4 + 3])
ground = px(1225, 12)
lit = sum(px(x, y) != ground for x in range(1230, 1250) for y in range(16, 32))
sys.exit(0 if lit > 20 else 1)
PY
    ok "$theme power: the bar shows the power icon at its right end"
}

listed_after() {
    local out
    for _ in $(seq 20); do
        out=$(shown_notices)
        grep -q "$1" <<<"$out" && break
        sleep 0.5
    done
    pace 1
    echo "$out"
}

cell_notice() {
    local theme=$1 shot=$work/notice.png base=$work/notice-base.png listed changed file
    ! ctl call notify state.get >/dev/null 2>&1 || fail "$theme notice: a notification server runs before the cell"
    start_notify "$shots/notify-$theme.log"
    rm -f "$work/stub/grim-fails"
    shoot "$base" || fail "$theme notice: the capture"
    keys Super+Print
    for _ in $(seq 40); do
        ls "$XDG_PICTURES_DIR"/Screenshots/*.png >/dev/null 2>&1 && break
        sleep 0.5
    done
    file=$(find "$XDG_PICTURES_DIR/Screenshots" -name "*.png" 2>/dev/null | head -n 1)
    [ -s "$file" ] || fail "$theme notice: Super+Print wrote no file"
    listed=$(listed_after "Screenshot saved")
    grep -q "Screenshot saved" <<<"$listed" || fail "$theme notice: no notice \"Screenshot saved\": $listed"
    grep -q "$file" <<<"$listed" || fail "$theme notice: the notice does not name the file $file: $listed"
    shoot "$shot" || fail "$theme notice: the capture"
    changed=$(differing "$base" "$shot" 900 650 1250 735)
    [ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-screenshot-saved-$theme.png"
    [ "$changed" -gt 500 ] || fail "$theme notice: nothing stands at the bottom right ($changed pixels differ)"
    ok "$theme notice: Super+Print saves $(basename "$file") and the notice names it"
    dismiss_notices
    settle
    : >"$work/stub/grim-fails"
    shoot "$base" || fail "$theme notice: the capture"
    keys Super+Print
    listed=$(listed_after "Screenshot failed")
    grep -q "Screenshot failed" <<<"$listed" || fail "$theme notice: no notice \"Screenshot failed\" when grim fails: $listed"
    grep -q "compositor refused the capture" <<<"$listed" || fail "$theme notice: the failure names no reason: $listed"
    shoot "$shot" || fail "$theme notice: the capture"
    changed=$(differing "$base" "$shot" 900 650 1250 735)
    [ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-screenshot-failed-$theme.png"
    [ "$changed" -gt 500 ] || fail "$theme notice: no failure stands at the bottom right ($changed pixels differ)"
    rm -f "$work/stub/grim-fails"
    ok "$theme notice: a failed screenshot says so"
    dismiss_notices
}

for theme in ${THEMES:-ink paper}; do
    start_round "$theme"
    cell_runs tooltip && cell_tooltip "$theme"
    cell_runs door && cell_door "$theme"
    cell_runs power && cell_power "$theme"
    cell_runs notice && cell_notice "$theme"
    end_round
done
ok "the honest states of tiling"
